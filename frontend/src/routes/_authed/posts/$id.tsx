/**
 * Post detail route — `/_authed/posts/$id`.
 *
 * Loader fetches a single post by numeric id via `getPostServerFn`.
 * The `id` path param is parsed as a positive integer; non-numeric
 * ids short-circuit to the not-found state without a backend round
 * trip.
 *
 * States:
 *   - loading                  → Spinner
 *   - not-found (404 / NaN)    → dedicated "not-found" panel
 *   - error (other 4xx/5xx)    → ErrorMessage block
 *   - happy path               → title, status, author, timestamps, body
 */
import { createFileRoute, Link, notFound } from '@tanstack/react-router'
import { Badge, Button, Card, ErrorMessage, Spinner } from '~/design-system'
import type { PostDto } from '~/lib/api/types'
import { getPostServerFn } from '~/server/posts'

export const Route = createFileRoute('/_authed/posts/$id')({
  // The path param is parsed with `parse` so the value arriving in the
  // loader is already a `number`. Non-numeric strings cause TanStack
  // Router to skip the loader and render the not-found surface.
  parseParams: (params) => ({
    id: Number.parseInt(params.id, 10),
  }),
  stringifyParams: ({ id }) => ({ id: String(id) }),
  validateSearch: () => ({}),
  loader: async ({ params }) => {
    if (!Number.isFinite(params.id) || params.id <= 0) {
      throw notFound()
    }
    try {
      return await getPostServerFn({ data: { id: params.id } })
    } catch (err) {
      // 404 from the backend is the canonical "not found" — rethrow as
      // `notFound()` so the framework renders the right surface.
      const message = err instanceof Error ? err.message : ''
      if (message.includes('404') || /not.?found/i.test(message)) {
        throw notFound()
      }
      // Anything else (network, 5xx) bubbles to `errorComponent`.
      throw err
    }
  },
  errorComponent: PostDetailError,
  notFoundComponent: PostDetailNotFound,
  component: PostDetailPage,
})

function PostDetailPage() {
  const post = Route.useLoaderData()
  return (
    <main
      data-ui="post-detail"
      data-ui-state="ready"
      className="mx-auto max-w-3xl px-6 py-8"
    >
      <Link
        to="/posts"
        className="text-text-muted hover:text-text mb-4 inline-block text-sm transition-colors"
        data-ui="post-detail-back"
      >
        ← All posts
      </Link>

      <Card>
        <header className="border-border-subtle flex flex-col gap-2 border-b pb-4">
          <div className="flex items-center gap-3">
            <h1 className="text-text flex-1 text-2xl font-semibold leading-tight">
              {post.title}
            </h1>
            <StatusBadge status={post.status} />
          </div>
          <div className="text-text-subtle flex flex-wrap items-center gap-x-3 gap-y-1 text-xs">
            {post.author ? <span>by {post.author.username}</span> : null}
            {post.slug ? (
              <span className="font-mono">/{post.slug}</span>
            ) : null}
            <time dateTime={post.created_at}>
              created {formatAbsolute(post.created_at)}
            </time>
            {post.updated_at && post.updated_at !== post.created_at ? (
              <time dateTime={post.updated_at}>
                updated {formatAbsolute(post.updated_at)}
              </time>
            ) : null}
            {post.published_at ? (
              <time dateTime={post.published_at}>
                published {formatAbsolute(post.published_at)}
              </time>
            ) : null}
            <span className="text-text-subtle/70">
              · {post.view_count} {post.view_count === 1 ? 'view' : 'views'}
            </span>
          </div>
        </header>

        {post.excerpt ? (
          <p className="text-text-muted mt-4 text-sm italic">
            {post.excerpt}
          </p>
        ) : null}

        <article
          className="text-text mt-4 whitespace-pre-wrap text-sm leading-relaxed"
          data-ui="post-detail-body"
        >
          {post.content}
        </article>

        <footer className="border-border-subtle mt-6 flex items-center gap-2 border-t pt-4">
          <Link
            to="/posts/$id/edit"
            params={{ id: String(post.id) }}
            data-ui="post-detail-edit"
          >
            <Button variant="secondary" size="sm">
              Edit
            </Button>
          </Link>
        </footer>
      </Card>
    </main>
  )
}

function PostDetailNotFound() {
  return (
    <main
      data-ui="post-detail"
      data-ui-state="not-found"
      className="mx-auto max-w-3xl px-6 py-12"
    >
      <Card>
        <h1 className="text-text text-xl font-semibold">Post not found</h1>
        <p className="text-text-muted mt-2 text-sm">
          The post you’re looking for doesn’t exist, was deleted, or you
          don’t have access to it.
        </p>
        <div className="mt-4">
          <Link to="/posts" data-ui="post-detail-notfound-back">
            <Button variant="secondary" size="sm">
              Back to posts
            </Button>
          </Link>
        </div>
      </Card>
    </main>
  )
}

function PostDetailError({ error }: { error: unknown }) {
  const message =
    error instanceof Error ? error.message : 'Unknown error loading the post.'
  return (
    <main
      data-ui="post-detail"
      data-ui-state="error"
      className="mx-auto max-w-3xl px-6 py-12"
    >
      <ErrorMessage block data-ui="post-detail-error">
        {message}
      </ErrorMessage>
      <div className="mt-4">
        <Link to="/posts" data-ui="post-detail-error-back">
          <Button variant="secondary" size="sm">
            Back to posts
          </Button>
        </Link>
      </div>
    </main>
  )
}

export function RoutePending() {
  return (
    <main
      data-ui="post-detail"
      data-ui-state="loading"
      className="mx-auto max-w-3xl px-6 py-12"
    >
      <Card>
        <div className="flex items-center gap-3 text-text-muted text-sm">
          <Spinner size="sm" />
          <span>Loading post…</span>
        </div>
      </Card>
    </main>
  )
}

function StatusBadge({ status }: { status: PostDto['status'] }) {
  const tone =
    status === 'published'
      ? 'success'
      : status === 'archived'
        ? 'warn'
        : status === 'deleted'
          ? 'danger'
          : 'neutral'
  const label = status.charAt(0).toUpperCase() + status.slice(1)
  return (
    <Badge tone={tone} data-ui="post-status">
      {label}
    </Badge>
  )
}

function formatAbsolute(iso: string): string {
  // Locale-formatted absolute date. The MVP intentionally avoids a date
  // lib; if we need relative + absolute + zoned later we can add one.
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  return d.toLocaleString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}
