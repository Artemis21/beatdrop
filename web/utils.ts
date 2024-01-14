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

// FIXME: if this isn't useful, remove (was for making the track bar animation smoother)
export function useTweened(target: number, time: number): number {
    const [anchor, setAnchor] = useState(target);
    const [tweened, setTweened] = useState(anchor);
    useEffect(() => {
        const FRAME_TIME = 33; // ~30 FPS
        const startTime = Date.now();
        const update = () => {
            const elapsed = Date.now() - startTime;
            if (elapsed >= time) {
                setAnchor(target);
                setTweened(target);
                clearInterval(interval);
            } else {
                setTweened(anchor + (target - anchor) * (elapsed / time));
            }
        };
        const interval = setInterval(update, FRAME_TIME);
        return () => clearInterval(interval);
    }, [anchor, target, time]);
    return tweened;
}
