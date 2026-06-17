/**
 * Posts list route — `/_authed/posts/`.
 *
 * Renders the current user's post list. Loader fetches the first page
 * from the backend via `listPostsServerFn`. The MVP does not implement
 * infinite scroll / load-more — pagination is a future concern.
 *
 * States:
 *   - loading (route-level pending)        → Spinner + token-styled panel
 *   - empty (no posts returned)            → EmptyState with "Create" CTA
 *   - error (loader threw)                 → ErrorMessage block
 *   - happy path                           → list of `PostListItem` cards
 */
import { createFileRoute, Link } from '@tanstack/react-router'
import { Badge, Button, Card, EmptyState, ErrorMessage, Spinner } from '~/design-system'
import type { PostDto } from '~/lib/api/types'
import { listPostsServerFn } from '~/server/posts'

export const Route = createFileRoute('/_authed/posts/')({
  loader: async () => {
    // The MVP grabs the first page (default limit=20) and does not
    // surface pagination controls. Returning the page wrapper so we
    // can grow into infinite-scroll later without a route shape
    // change.
    return await listPostsServerFn({ data: {} })
  },
  component: PostsListPage,
})

function PostsListPage() {
  // `useLoaderData` is the typed hook. TanStack Router infers the
  // return type from the loader above.
  const page = Route.useLoaderData()

  if (!page) {
    // Should be unreachable — `loader` always returns a page — but the
    // type-system widens to `unknown` without this guard.
    return <PostsError message="No data returned from the server." />
  }

  const posts = page.data

  if (posts.length === 0) {
    return (
      <main
        data-ui="posts-list"
        data-ui-state="empty"
        className="mx-auto max-w-3xl px-6 py-12"
      >
        <EmptyState
          icon={<DocumentIcon />}
          title="No posts yet"
          description="Create the first post to see it appear here."
          action={
            <Link to="/posts/new" data-ui="posts-empty-create">
              <Button variant="primary">Create post</Button>
            </Link>
          }
        />
      </main>
    )
  }

  return (
    <main
      data-ui="posts-list"
      data-ui-state="ready"
      className="mx-auto max-w-3xl px-6 py-8"
    >
      <header className="mb-6 flex items-baseline justify-between">
        <div>
          <h1 className="text-text text-2xl font-semibold leading-tight">
            Posts
          </h1>
          <p className="text-text-muted mt-1 text-sm">
            {posts.length} {posts.length === 1 ? 'post' : 'posts'}
          </p>
        </div>
        <Link to="/posts/new" data-ui="posts-list-create">
          <Button variant="primary" size="sm">
            New post
          </Button>
        </Link>
      </header>

      <ul className="flex flex-col gap-3" data-ui="post-list">
        {posts.map((post) => (
          <li key={post.id} data-ui="post-list-item">
            <PostListItem post={post} />
          </li>
        ))}
      </ul>
    </main>
  )
}

function PostListItem({ post }: { post: PostDto }) {
  // The MVP shows the bare minimum: title, status badge, and a
  // relative-time stamp. Excerpt / body are deferred to the detail
  // page. `view_count` is intentionally not surfaced prominently
  // because the backend increments it on every detail fetch.
  return (
    <Link
      to="/posts/$id"
      params={{ id: post.id }}
      className="block focus:outline-none"
      data-ui="post-list-link"
    >
      <Card className="hover:bg-surface-2 transition-colors">
        <div className="flex items-center gap-3">
          <h2 className="text-text flex-1 text-base font-medium leading-snug">
            {post.title}
          </h2>
          <StatusBadge status={post.status} />
        </div>
        <div className="text-text-subtle mt-2 flex items-center gap-3 text-xs">
          {post.author ? (
            <span>by {post.author.username}</span>
          ) : null}
          <time dateTime={post.created_at}>
            {formatRelative(post.created_at)}
          </time>
        </div>
      </Card>
    </Link>
  )
}

function StatusBadge({ status }: { status: PostDto['status'] }) {
  // Map backend status → design-system Badge tone.
  const tone =
    status === 'published'
      ? 'success'
      : status === 'archived'
        ? 'warn'
        : 'neutral'
  const label =
    status.charAt(0).toUpperCase() + status.slice(1)
  return (
    <Badge tone={tone} data-ui="post-status">
      {label}
    </Badge>
  )
}

function PostsError({ message }: { message: string }) {
  return (
    <main
      data-ui="posts-list"
      data-ui-state="error"
      className="mx-auto max-w-3xl px-6 py-12"
    >
      <ErrorMessage block data-ui="posts-list-error">
        {message}
      </ErrorMessage>
    </main>
  )
}

/**
 * Route-level pending surface. TanStack Router renders this when the
 * loader is still in flight. The shell header stays visible because it
 * lives in the `_authed` layout above this route.
 */
export function RoutePending() {
  return (
    <main
      data-ui="posts-list"
      data-ui-state="loading"
      className="mx-auto max-w-3xl px-6 py-12"
    >
      <Card>
        <div className="flex items-center gap-3 text-text-muted text-sm">
          <Spinner size="sm" />
          <span>Loading posts…</span>
        </div>
      </Card>
    </main>
  )
}

// ---- helpers --------------------------------------------------------------

function formatRelative(iso: string): string {
  // Lazy import to avoid a hard dep on a date lib in the route file —
  // `Intl.RelativeTimeFormat` is in every modern runtime and gives
  // a Linear-style "2h ago" feel without adding a dep.
  const then = new Date(iso).getTime()
  if (Number.isNaN(then)) return iso
  const diffSec = Math.round((then - Date.now()) / 1000)
  const abs = Math.abs(diffSec)
  const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' })
  if (abs < 60) return rtf.format(diffSec, 'second')
  if (abs < 3600) return rtf.format(Math.round(diffSec / 60), 'minute')
  if (abs < 86400) return rtf.format(Math.round(diffSec / 3600), 'hour')
  if (abs < 86400 * 30)
    return rtf.format(Math.round(diffSec / 86400), 'day')
  if (abs < 86400 * 365)
    return rtf.format(Math.round(diffSec / (86400 * 30)), 'month')
  return rtf.format(Math.round(diffSec / (86400 * 365)), 'year')
}

function DocumentIcon() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      className="h-8 w-8"
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z"
      />
    </svg>
  )
}

// Re-export for clarity in dev tools.
export { formatRelative }
