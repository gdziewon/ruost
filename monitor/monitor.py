import time
import json
import os
import logging
from datetime import datetime, timezone
from tenacity import retry, stop_after_attempt, wait_fixed, before_sleep_log
import requests_unixsocket

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

SAMPLE_INTERVAL = 10
LOG_DIR = "/logs"
CONTAINER_NAME = "ruost"
API_URL = "http+unix://%2Fpodman.sock/v1.40/containers/{}/stats?stream=false"

session = requests_unixsocket.Session()

def calculate_cpu_percent(stats):
    try:
        cpu_stats = stats.get("cpu_stats", {})
        precpu_stats = stats.get("precpu_stats", {})
        
        # get CPU usage deltas
        cpu_usage = cpu_stats.get("cpu_usage", {})
        precpu_usage = precpu_stats.get("cpu_usage", {})
        
        cpu_delta = cpu_usage.get("total_usage", 0) - precpu_usage.get("total_usage", 0)
        system_delta = cpu_stats.get("system_cpu_usage", 0) - precpu_stats.get("system_cpu_usage", 0)
        
        # to handle different Podman versions
        online_cpus = cpu_stats.get("online_cpus") or \
                     cpu_stats.get("cpu_usage", {}).get("percpu_usage", [0]) and \
                     len(cpu_stats.get("cpu_usage", {}).get("percpu_usage", [0])) or 1

        if system_delta <= 0 or cpu_delta <= 0:
            return 0.0
            
        return round((cpu_delta / system_delta) * 100.0 * online_cpus, 2)
        
    except KeyError as e:
        logger.debug(f"Missing CPU metric: {e}")
        return 0.0
    except ZeroDivisionError:
        return 0.0
    except Exception as e:
        logger.error(f"CPU calculation error: {e}", exc_info=True)
        return 0.0

@retry(
    stop=stop_after_attempt(5),
    wait=wait_fixed(2),
    before_sleep=before_sleep_log(logger, logging.WARNING)
)
def get_stats(container_name):
    try:
        start_time = time.time()
        response = session.get(API_URL.format(container_name), timeout=5)
        response.raise_for_status()
        
        stats = response.json()
        
        return {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "cpu_percent": calculate_cpu_percent(stats),
            "memory": {
                "usage": stats.get("memory_stats", {}).get("usage", 0),
                "limit": stats.get("memory_stats", {}).get("limit", 0),
                "usage_percent": round(
                    stats.get("memory_stats", {}).get("usage", 0) / 
                    (stats.get("memory_stats", {}).get("limit", 1) or 1) * 100, 2
                )
            },
            "networks": stats.get("networks", {}),
            "duration_ms": round((time.time() - start_time) * 1000, 2)
        }
        
    except Exception as e:
        logger.error(f"Error getting stats: {e}")
        raise

def main():
    os.makedirs(LOG_DIR, exist_ok=True)
    log_path = os.path.join(LOG_DIR, "stats.jsonl")
    
    logger.info(f"Starting monitoring for container {CONTAINER_NAME}")
    
    try:
        with open(log_path, "a", buffering=1) as f:
            while True:
                try:
                    stats = get_stats(CONTAINER_NAME)
                    f.write(json.dumps(stats) + "\n")
                except Exception as e:
                    logger.error(f"Failed to collect stats: {e}")
                    error_entry = {
                        "timestamp": datetime.now(timezone.utc).isoformat(),
                        "error": str(e),
                        "cpu_percent": 0.0,
                        "memory": {"usage": 0, "limit": 0, "usage_percent": 0}
                    }
                    f.write(json.dumps(error_entry) + "\n")
                
                time.sleep(SAMPLE_INTERVAL)
                
    except KeyboardInterrupt:
        logger.info("Monitoring stopped by user")

if __name__ == "__main__":
    main()