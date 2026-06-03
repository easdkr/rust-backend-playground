/**
 * Create-post route — `/_authed/posts/new`.
 *
 * Renders a centered form with the four editable fields from the
 * backend's `CreatePostCmd` (title, content, excerpt, slug). On
 * success it navigates to the new post's detail page so the user
 * sees what they just created.
 *
 * Client-side validation mirrors the backend's `UpdatePostCmd` rules
 * (see `crates/application/src/post/dto.rs`): required title +
 * content, optional excerpt capped at 500 chars, optional slug
 * constrained to `[A-Za-z0-9._-]{1,140}`. The server is the source
 * of truth — these checks exist to catch typos before a round trip.
 */
import { useState, type FormEvent } from 'react'
import {
  createFileRoute,
  redirect,
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
import { getCurrentSessionServerFn } from '~/server/auth'
import { createPostServerFn } from '~/server/posts'

export const Route = createFileRoute('/_authed/posts/new')({
  beforeLoad: async () => {
    // The auth-guard layout already checked the session, but the MVP
    // additionally requires `PostCreate` permission per the backend
    // (`crates/api/src/http/auth/mod.rs` RBAC). For now, the in-memory
    // session only stores the username; a real /me endpoint is out
    // of scope, so we simply require *some* session (the guard above
    // already enforces this). This is a no-op placeholder kept for
    // future role-aware UI.
    const session = await getCurrentSessionServerFn()
    if (session === null) {
      throw redirect({ to: '/login', search: { redirect: '/posts/new' } })
    }
  },
  component: NewPostPage,
})

type FieldErrors = Partial<Record<'title' | 'content' | 'excerpt' | 'slug', string>>

const TITLE_MAX = 200
const EXCERPT_MAX = 500
const SLUG_MAX = 140
const SLUG_RE = /^[A-Za-z0-9._-]+$/

function NewPostPage() {
  const navigate = useNavigate()
  const [title, setTitle] = useState('')
  const [content, setContent] = useState('')
  const [excerpt, setExcerpt] = useState('')
  const [slug, setSlug] = useState('')
  const [fieldErrors, setFieldErrors] = useState<FieldErrors>({})
  const [formError, setFormError] = useState<string | null>(null)
  const [submitting, setSubmitting] = useState(false)

  function clientValidate(): FieldErrors {
    const errs: FieldErrors = {}
    if (title.trim().length === 0) errs.title = 'Title is required.'
    else if (title.length > TITLE_MAX)
      errs.title = `Title must be ${TITLE_MAX} characters or fewer.`
    if (content.trim().length === 0) errs.content = 'Content is required.'
    if (excerpt.length > EXCERPT_MAX)
      errs.excerpt = `Excerpt must be ${EXCERPT_MAX} characters or fewer.`
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
      const payload = {
        title: title.trim(),
        content,
        ...(excerpt.trim().length > 0 ? { excerpt: excerpt.trim() } : {}),
        ...(slug.trim().length > 0 ? { slug: slug.trim() } : {}),
      }
      const post = await createPostServerFn({ data: payload })
      await navigate({
        to: '/posts/$id',
        params: { id: String(post.id) },
      })
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Could not create the post.'
      setFormError(message)
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <main
      data-ui="post-new"
      className="mx-auto max-w-2xl px-6 py-8"
    >
      <header className="mb-6">
        <h1 className="text-text text-2xl font-semibold leading-tight">
          New post
        </h1>
        <p className="text-text-muted mt-1 text-sm">
          Drafts are private to you. Publishing happens elsewhere in the
          app — out of MVP scope.
        </p>
      </header>

      <Card>
        <form
          onSubmit={onSubmit}
          noValidate
          className="flex flex-col gap-4"
          data-ui="post-new-form"
        >
          {formError !== null ? (
            <ErrorMessage data-ui="post-new-error" role="alert">
              {formError}
            </ErrorMessage>
          ) : null}

          <Field
            label="Title"
            htmlFor="post-new-title"
            hint={`Up to ${TITLE_MAX} characters.`}
            error={fieldErrors.title}
            invalid={Boolean(fieldErrors.title)}
          >
            <Input
              id="post-new-title"
              name="title"
              type="text"
              required
              autoFocus
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              disabled={submitting}
              invalid={Boolean(fieldErrors.title)}
              data-ui="post-new-title"
            />
          </Field>

          <Field
            label="Slug"
            htmlFor="post-new-slug"
            hint="Optional. URL-friendly identifier; letters, digits, dot, underscore, hyphen."
            error={fieldErrors.slug}
            invalid={Boolean(fieldErrors.slug)}
          >
            <Input
              id="post-new-slug"
              name="slug"
              type="text"
              value={slug}
              onChange={(e) => setSlug(e.target.value)}
              disabled={submitting}
              invalid={Boolean(fieldErrors.slug)}
              data-ui="post-new-slug"
            />
          </Field>

          <Field
            label="Excerpt"
            htmlFor="post-new-excerpt"
            hint={`Optional. Up to ${EXCERPT_MAX} characters.`}
            error={fieldErrors.excerpt}
            invalid={Boolean(fieldErrors.excerpt)}
          >
            <Textarea
              id="post-new-excerpt"
              name="excerpt"
              rows={3}
              value={excerpt}
              onChange={(e) => setExcerpt(e.target.value)}
              disabled={submitting}
              invalid={Boolean(fieldErrors.excerpt)}
              data-ui="post-new-excerpt"
            />
          </Field>

          <Field
            label="Content"
            htmlFor="post-new-content"
            hint="The body of the post."
            error={fieldErrors.content}
            invalid={Boolean(fieldErrors.content)}
          >
            <Textarea
              id="post-new-content"
              name="content"
              rows={10}
              required
              value={content}
              onChange={(e) => setContent(e.target.value)}
              disabled={submitting}
              invalid={Boolean(fieldErrors.content)}
              data-ui="post-new-content"
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
              {submitting ? 'Creating…' : 'Create post'}
            </Button>
            <Button
              type="button"
              variant="ghost"
              disabled={submitting}
              onClick={() => navigate({ to: '/posts' })}
              data-ui="post-cancel"
            >
              Cancel
            </Button>
            {submitting ? <Spinner size="sm" /> : null}
          </div>
        </form>
      </Card>
    </main>
  )
}
