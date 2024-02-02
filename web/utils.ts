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

/**
 * Generate `classNames` of the form `base base--mod1 base--mod2` where `base`
 * is the first parameter and `mod1` and `mod2` are keys of the second
 * parameter. Modifiers will only be included if the corresponding value is
 * truthy.
 */
export function classModifiers(base: string, modifiers: Record<string, unknown>): string {
    let className = base;
    for (const [modifier, enabled] of Object.entries(modifiers)) {
        if (enabled) className += ` ${base}--${modifier}`;
    }
    return className;
}
