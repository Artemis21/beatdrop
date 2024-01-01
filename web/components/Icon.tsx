export function Icon({ icon, className }: { icon: String; className?: String }) {
    return <i className={`fa-solid fa-fw fa-${icon} ${className}`}></i>;
}
