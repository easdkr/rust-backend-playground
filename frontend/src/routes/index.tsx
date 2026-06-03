import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/')({
  head: () => ({
    meta: [
      { title: 'Home — rust-backend-playground' },
      {
        name: 'description',
        content: 'Placeholder home page for the rust-backend-playground TanStack Start app.',
      },
    ],
  }),
  component: HomePage,
})

function HomePage() {
  return (
    <main className="mx-auto max-w-2xl px-6 py-12">
      <header className="mb-8">
        <p className="text-text-subtle text-xs font-medium uppercase tracking-wider">
          rust-backend-playground
        </p>
        <h1 className="text-text text-3xl font-semibold leading-tight">
          Frontend scaffold
        </h1>
        <p className="text-text-muted text-sm">
          TanStack Start + React 19 + Vite 8, styled with a token-driven
          Tailwind v4 theme.
        </p>
      </header>

      <section className="bg-surface border-border-subtle rounded-md border p-6 shadow-xs">
        <h2 className="text-text text-lg font-medium">Design system wired</h2>
        <p className="text-text-muted text-sm">
          This panel is composed entirely from <code className="text-text font-mono text-xs">src/design-system/tokens.css</code> —
          <span className="text-text-subtle"> surfaces, borders, radius, and shadow are all tokens.</span>
        </p>

        <div className="mt-5 flex flex-wrap items-center gap-3">
          <button
            type="button"
            className="bg-accent text-accent-fg rounded-pill px-4 py-1.5 text-sm font-medium transition-colors"
          >
            Primary action
          </button>
          <button
            type="button"
            className="border-border text-text rounded-pill border px-4 py-1.5 text-sm font-medium transition-colors"
          >
            Secondary
          </button>
          <span className="text-text-subtle text-xs">
            Hover/focus shows the accent ring.
          </span>
        </div>
      </section>

      <section className="bg-surface border-border-subtle mt-4 rounded-md border p-6 shadow-xs">
        <h2 className="text-text text-lg font-medium">Token preview</h2>
        <dl className="text-sm">
          <div className="flex items-center justify-between border-border-subtle border-b py-2">
            <dt className="text-text-muted">Surfaces</dt>
            <dd className="text-text font-mono text-xs">bg · surface · surface-2</dd>
          </div>
          <div className="flex items-center justify-between border-border-subtle border-b py-2">
            <dt className="text-text-muted">Text</dt>
            <dd className="text-text font-mono text-xs">text · text-muted · text-subtle</dd>
          </div>
          <div className="flex items-center justify-between border-border-subtle border-b py-2">
            <dt className="text-text-muted">Borders</dt>
            <dd className="text-text font-mono text-xs">border · border-subtle</dd>
          </div>
          <div className="flex items-center justify-between py-2">
            <dt className="text-text-muted">Radius</dt>
            <dd className="text-text font-mono text-xs">sm · md · lg · pill</dd>
          </div>
        </dl>
      </section>
    </main>
  )
}
