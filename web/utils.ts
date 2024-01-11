import { useState, useEffect } from "react";

export function useThrottled<T>(value: T, delay: number): T {
    const [throttled, setThrottled] = useState(value);
    const [unthrottleTime, setUnthrottleTime] = useState(0);
    useEffect(() => {
        const now = Date.now();
        if (unthrottleTime <= now) {
            setThrottled(value);
            setUnthrottleTime(now + delay);
        } else {
            const timeout = setTimeout(() => {
                setThrottled(value);
                setUnthrottleTime(Date.now() + delay);
            }, unthrottleTime - now);
            return () => clearTimeout(timeout);
        }
    }, [value, delay, unthrottleTime]);
    return throttled;
}
