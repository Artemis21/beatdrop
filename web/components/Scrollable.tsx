import { ReactNode, useEffect, useRef, useState } from "react";

export function Scrollable({ children }: { children: ReactNode }) {
    const top = useRef<HTMLDivElement | null>(null);
    const bottom = useRef<HTMLDivElement | null>(null);
    const [scrollUp, setScrollUp] = useState(false);
    const [scrollDown, setScrollDown] = useState(false);
    useEffect(() => {
        if (!top.current || !bottom.current) return;
        const observer = new IntersectionObserver(entries => {
            entries.forEach(entry => {
                if (entry.target === top.current) {
                    setScrollUp(!entry.isIntersecting);
                } else if (entry.target === bottom.current) {
                    setScrollDown(!entry.isIntersecting);
                }
            });
        });
        observer.observe(top.current);
        observer.observe(bottom.current);
    }, [top, bottom, setScrollDown, setScrollUp]);
    let className = "scrollable";
    if (scrollUp) className += " scrollable--scroll-up";
    if (scrollDown) className += " scrollable--scroll-down";
    return (
        <div className={className}>
            <div className="scrollable__trigger" ref={top}></div>
            {children}
            <div className="scrollable__trigger" ref={bottom}></div>
        </div>
    );
}
