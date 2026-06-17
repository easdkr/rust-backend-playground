/**
 * Edit-post route — `/_authed/posts/$id/edit`.
 *
 * Loads the existing post via the `getPostServerFn` and renders a
 * form pre-populated with its fields. Submitting calls
 * `updatePostServerFn`; on success we navigate to the detail page.
 *
 * Differences from the create form:
 *   - `loader` pre-fills initial state (so the form is a controlled
 *     edit, not a re-create)
 *   - All fields are optional in the payload — the backend treats
 *     omitted fields as "leave unchanged"
 *   - Client validation is softer: we only check the *character*
 *     caps and slug pattern, not "required" (since each field is
 *     optional on update)
 *   - The submit button label is "Save changes"
 */
import { useEffect, useState, type FormEvent } from 'react'
import {
  createFileRoute,
  Link,
  notFound,
  useNavigate,
} from '@tanstack/react-router'
import {
  Button,
  Card,
  ErrorMessage,
  Field,
  Input,
  Spinner,
  Textarea,
} from '~/design-system'
import { getPostServerFn, updatePostServerFn } from '~/server/posts'

export const Route = createFileRoute('/_authed/posts/$id_/edit')({
  parseParams: (params) => ({
    id: Number.parseInt(params.id, 10),
  }),
  stringifyParams: ({ id }) => ({ id: String(id) }),
  validateSearch: () => ({}),
  loader: async ({ params }) => {
    if (!Number.isFinite(params.id) || params.id <= 0) {
      throw notFound()
    }
    return await getPostServerFn({ data: { id: params.id } })
  },
  errorComponent: EditPostError,
  notFoundComponent: EditPostNotFound,
  component: EditPostPage,
})

type FieldErrors = Partial<Record<'title' | 'content' | 'excerpt' | 'slug', string>>

const TITLE_MAX = 200
const EXCERPT_MAX = 500
const SLUG_MAX = 140
const SLUG_RE = /^[A-Za-z0-9._-]+$/

function EditPostPage() {
  const post = Route.useLoaderData()
  const navigate = useNavigate()

  // Local form state. `useEffect` reseeds when the loader data changes
  // (e.g. if the user navigates between two edit URLs without
  // unmounting the page). Without it, React would warn on the initial
  // controlled->uncontrolled transition.
  const [title, setTitle] = useState(post.title)
  const [content, setContent] = useState(post.content)
  const [excerpt, setExcerpt] = useState(post.excerpt ?? '')
  const [slug, setSlug] = useState(post.slug ?? '')
  const [fieldErrors, setFieldErrors] = useState<FieldErrors>({})
  const [formError, setFormError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  useEffect(() => {
    setTitle(post.title)
    setContent(post.content)
    setExcerpt(post.excerpt ?? '')
    setSlug(post.slug ?? '')
    setFieldErrors({})
    setFormError(null)
  }, [post.id, post.title, post.content, post.excerpt, post.slug])

  function clientValidate(): FieldErrors {
    const errs: FieldErrors = {}
    // Title: only enforce the cap if the user is actually changing it
    // to something non-empty. On update the backend treats empty as
    // "no change", so we let the form submit through.
    if (title.length > TITLE_MAX) {
      errs.title = `Title must be ${TITLE_MAX} characters or fewer.`
    }
    if (excerpt.length > EXCERPT_MAX) {
      errs.excerpt = `Excerpt must be ${EXCERPT_MAX} characters or fewer.`
    }
    if (slug.length > 0) {
      if (slug.length > SLUG_MAX) {
        errs.slug = `Slug must be ${SLUG_MAX} characters or fewer.`
      } else if (!SLUG_RE.test(slug)) {
        errs.slug = 'Slug may contain letters, digits, dot, underscore, and hyphen only.'
      } else if (slug.startsWith('-') || slug.endsWith('-')) {
        errs.slug = 'Slug may not start or end with a hyphen.'
      }
    }
    return errs
  }

  async function onSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setFormError(null)
    const errs = clientValidate()
    setFieldErrors(errs)
    if (Object.keys(errs).length > 0) return

    setSubmitting(true)
    try {
      // Build the partial update payload. The backend's
      // `UpdatePostCmd` has every field as optional, so we send
      // only the fields the user actually changed.
      const cmd: Record<string, string> = {}
      if (title !== post.title) cmd.title = title
      if (content !== post.content) cmd.content = content
      const trimmedExcerpt = excerpt.trim()
      if (trimmedExcerpt !== (post.excerpt ?? '')) {
        if (trimmedExcerpt.length > 0) cmd.excerpt = trimmedExcerpt
        else cmd.excerpt = '' // explicit clear
      }
      const trimmedSlug = slug.trim()
      if (trimmedSlug !== (post.slug ?? '')) {
        if (trimmedSlug.length > 0) cmd.slug = trimmedSlug
        else cmd.slug = '' // explicit clear
      }

      // If the user changed nothing, just bounce them back to the
      // detail page without making a no-op request.
      if (Object.keys(cmd).length === 0) {
        await navigate({ to: '/posts/$id', params: { id: post.id } })
        return
      }

      await updatePostServerFn({
        data: { id: post.id, cmd: cmd as never },
      })
      await navigate({ to: '/posts/$id', params: { id: post.id } })
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Could not save the post.'
      setFormError(message)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <main
      data-ui="post-edit"
      className="mx-auto max-w-2xl px-6 py-8"
    >
      <Link
        to="/posts/$id"
        params={{ id: post.id }}
        className="text-text-muted hover:text-text mb-4 inline-block text-sm transition-colors"
        data-ui="post-edit-back"
      >
        ← Back to post
      </Link>

      <header className="mb-6">
        <h1 className="text-text text-2xl font-semibold leading-tight">
          Edit post
        </h1>
        <p className="text-text-muted mt-1 text-sm">
          Leave a field empty to clear it. Submit only what you want
          to change.
        </p>
      </header>

      <Card>
        <form
          onSubmit={onSubmit}
          noValidate
          className="flex flex-col gap-4"
          data-ui="post-edit-form"
        >
          {formError !== null ? (
            <ErrorMessage data-ui="post-edit-error" role="alert">
              {formError}
            </ErrorMessage>
          ) : null}

          <Field
            label="Title"
            htmlFor="post-edit-title"
            hint={`Up to ${TITLE_MAX} characters.`}
            error={fieldErrors.title}
            invalid={Boolean(fieldErrors.title)}
          >
            <Input
              id="post-edit-title"
              name="title"
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              disabled={submitting}
              invalid={Boolean(fieldErrors.title)}
              data-ui="post-edit-title"
            />
          </Field>

          <Field
            label="Slug"
            htmlFor="post-edit-slug"
            hint="Optional. URL-friendly identifier."
            error={fieldErrors.slug}
            invalid={Boolean(fieldErrors.slug)}
          >
            <Input
              id="post-edit-slug"
              name="slug"
              type="text"
              value={slug}
              onChange={(e) => setSlug(e.target.value)}
              disabled={submitting}
              invalid={Boolean(fieldErrors.slug)}
              data-ui="post-edit-slug"
            />
          </Field>

          <Field
            label="Excerpt"
            htmlFor="post-edit-excerpt"
            hint={`Optional. Up to ${EXCERPT_MAX} characters.`}
            error={fieldErrors.excerpt}
            invalid={Boolean(fieldErrors.excerpt)}
          >
            <Textarea
              id="post-edit-excerpt"
              name="excerpt"
              rows={3}
              value={excerpt}
              onChange={(e) => setExcerpt(e.target.value)}
              disabled={submitting}
              invalid={Boolean(fieldErrors.excerpt)}
              data-ui="post-edit-excerpt"
            />
          </Field>

          <Field
            label="Content"
            htmlFor="post-edit-content"
            hint="The body of the post."
            error={fieldErrors.content}
            invalid={Boolean(fieldErrors.content)}
          >
            <Textarea
              id="post-edit-content"
              name="content"
              rows={10}
              value={content}
              onChange={(e) => setContent(e.target.value)}
              disabled={submitting}
              invalid={Boolean(fieldErrors.content)}
              data-ui="post-edit-content"
            />
          </Field>

          <div className="mt-2 flex items-center gap-3">
            <Button
              type="submit"
              variant="primary"
              loading={submitting}
              disabled={submitting}
              data-ui="post-save"
            >
              {submitting ? 'Saving…' : 'Save changes'}
            </Button>
            <Link
              to="/posts/$id"
              params={{ id: post.id }}
              data-ui="post-cancel"
            >
              <Button type="button" variant="ghost" disabled={submitting}>
                Cancel
              </Button>
            </Link>
            {submitting ? <Spinner size="sm" /> : null}
          </div>
        </form>
      </Card>
    </main>
  )
}

function EditPostNotFound() {
  return (
    <main
      data-ui="post-edit"
      data-ui-state="not-found"
      className="mx-auto max-w-2xl px-6 py-12"
    >
      <Card>
        <h1 className="text-text text-xl font-semibold">Post not found</h1>
        <p className="text-text-muted mt-2 text-sm">
          The post you’re trying to edit doesn’t exist or you don’t
          have permission to modify it.
        </p>
        <div className="mt-4">
          <Link to="/posts" data-ui="post-edit-notfound-back">
            <Button variant="secondary" size="sm">
              Back to posts
            </Button>
          </Link>
        </div>
      </Card>
    </main>
  )
}

function EditPostError({ error }: { error: unknown }) {
  const message =
    error instanceof Error
      ? error.message
      : 'Unknown error loading the post.'
  return (
    <main
      data-ui="post-edit"
      data-ui-state="error"
      className="mx-auto max-w-2xl px-6 py-12"
    >
      <ErrorMessage block data-ui="post-edit-error">
        {message}
      </ErrorMessage>
      <div className="mt-4">
        <Link to="/posts" data-ui="post-edit-error-back">
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
      data-ui="post-edit"
      data-ui-state="loading"
      className="mx-auto max-w-2xl px-6 py-12"
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
