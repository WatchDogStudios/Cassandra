export function GridBackground() {
  return (
    <div className="absolute inset-0 z-0 overflow-hidden">
      {/* Grid lines */}
      <div
        className="absolute inset-0"
        style={{
          backgroundImage: `
            linear-gradient(var(--grid-color) 1px, transparent 1px),
            linear-gradient(90deg, var(--grid-color) 1px, transparent 1px)
          `,
          backgroundSize: "60px 60px",
          maskImage: "radial-gradient(ellipse 80% 50% at 50% 100%, black 0%, black 40%, transparent 70%)",
          WebkitMaskImage: "radial-gradient(ellipse 80% 50% at 50% 100%, black 0%, black 40%, transparent 70%)",
        }}
      />
      {/* Glow effect */}
      <div
        className="absolute inset-0"
        style={{
          background: "radial-gradient(ellipse 100% 60% at 50% 100%, var(--grid-glow), transparent 60%)",
        }}
      />
    </div>
  )
}
