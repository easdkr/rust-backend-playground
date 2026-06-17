import { useEffect, useState, type FormEvent } from 'react'
import { useRouter } from '@tanstack/react-router'

import { Button, Card, ErrorMessage, Field, Textarea } from '~/design-system'
import type {
  CommentDto,
  CommentThreadDto,
  CreateCommentCmd,
  PostDto,
  UpdateCommentCmd,
  UserDto,
} from '~/lib/api/types'
import {
  createCommentServerFn,
  createReplyServerFn,
  deleteCommentServerFn,
  listCommentsServerFn,
  updateCommentServerFn,
} from '~/server/comments'

const CONTENT_MAX = 2000

interface CommentSectionProps {
  post: PostDto
  currentUser: UserDto
}

export function CommentSection({ post, currentUser }: CommentSectionProps) {
  const router = useRouter()

  const [threads, setThreads] = useState<CommentThreadDto[]>(post.comments)
  const [hasMore, setHasMore] = useState<boolean>(post.has_more_comments)
  const [loadingMore, setLoadingMore] = useState(false)

  useEffect(() => {
    setThreads(post.comments)
    setHasMore(post.has_more_comments)
  }, [post.comments, post.has_more_comments])

  const [rootContent, setRootContent] = useState('')
  const [rootError, setRootError] = useState<string | null>(null)
  const [rootSubmitting, setRootSubmitting] = useState(false)

  const [replyingToId, setReplyingToId] = useState<number | null>(null)
  const [replyContent, setReplyContent] = useState('')
  const [replyError, setReplyError] = useState<string | null>(null)
  const [replySubmitting, setReplySubmitting] = useState(false)

  const [editingId, setEditingId] = useState<number | null>(null)
  const [editContent, setEditContent] = useState('')
  const [editError, setEditError] = useState<string | null>(null)
  const [editSubmitting, setEditSubmitting] = useState(false)

  const [globalError, setGlobalError] = useState<string | null>(null)

  function validateContent(content: string): string | null {
    if (content.trim().length === 0) return 'Comment cannot be empty.'
    if (content.length > CONTENT_MAX) return `Comment must be ${CONTENT_MAX} characters or fewer.`
    return null
  }

  async function invalidatePost() {
    await router.invalidate()
  }

  async function handleLoadMore() {
    setLoadingMore(true)
    setGlobalError(null)
    try {
      const limit = Math.min(post.comment_count, 100)
      const page = await listCommentsServerFn({
        data: { post_id: post.id, query: { limit } },
      })
      // The backend list endpoint returns roots in ascending id order (oldest
      // first). Reverse so the newest roots appear first, matching the
      // `recent_threads` order returned with the post detail.
      setThreads([...page.data].reverse())
      setHasMore(page.has_more && page.data.length < post.comment_count)
    } catch (err) {
      setGlobalError(err instanceof Error ? err.message : 'Could not load comments.')
    } finally {
      setLoadingMore(false)
    }
  }

  async function handleCreateRoot(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const error = validateContent(rootContent)
    setRootError(error)
    if (error !== null) return

    setRootSubmitting(true)
    setGlobalError(null)
    try {
      const cmd: CreateCommentCmd = { content: rootContent.trim() }
      await createCommentServerFn({ data: { post_id: post.id, cmd } })
      setRootContent('')
      await invalidatePost()
    } catch (err) {
      setRootError(err instanceof Error ? err.message : 'Could not post comment.')
    } finally {
      setRootSubmitting(false)
    }
  }

  async function handleReply(parentCommentId: number) {
    const error = validateContent(replyContent)
    setReplyError(error)
    if (error !== null) return

    setReplySubmitting(true)
    setGlobalError(null)
    try {
      const cmd: CreateCommentCmd = { content: replyContent.trim() }
      await createReplyServerFn({
        data: { post_id: post.id, parent_comment_id: parentCommentId, cmd },
      })
      setReplyContent('')
      setReplyingToId(null)
      await invalidatePost()
    } catch (err) {
      setReplyError(err instanceof Error ? err.message : 'Could not post reply.')
    } finally {
      setReplySubmitting(false)
    }
  }

  async function handleUpdate(commentId: number) {
    const error = validateContent(editContent)
    setEditError(error)
    if (error !== null) return

    setEditSubmitting(true)
    setGlobalError(null)
    try {
      const cmd: UpdateCommentCmd = { content: editContent.trim() }
      await updateCommentServerFn({
        data: { post_id: post.id, comment_id: commentId, cmd },
      })
      setEditingId(null)
      setEditContent('')
      await invalidatePost()
    } catch (err) {
      setEditError(err instanceof Error ? err.message : 'Could not update comment.')
    } finally {
      setEditSubmitting(false)
    }
  }

  async function handleDelete(commentId: number) {
    if (!confirm('Delete this comment?')) return
    setGlobalError(null)
    try {
      await deleteCommentServerFn({ data: { post_id: post.id, comment_id: commentId } })
      await invalidatePost()
    } catch (err) {
      setGlobalError(err instanceof Error ? err.message : 'Could not delete comment.')
    }
  }

  function startEdit(comment: CommentDto) {
    setEditingId(comment.id)
    setEditContent(comment.content)
    setEditError(null)
  }

  function startReply(comment: CommentDto) {
    setReplyingToId(comment.id)
    setReplyContent('')
    setReplyError(null)
  }

  function cancelReply() {
    setReplyingToId(null)
    setReplyContent('')
    setReplyError(null)
  }

  function cancelEdit() {
    setEditingId(null)
    setEditContent('')
    setEditError(null)
  }

  const canPostRoot = currentUser !== undefined

  return (
    <section className="mt-8" data-ui="comment-section">
      <div className="mb-4 flex items-baseline justify-between">
        <h2 className="text-text text-lg font-semibold">
          Comments
          {post.comment_count > 0 ? (
            <span className="text-text-subtle ml-2 text-sm font-normal">
              ({post.comment_count})
            </span>
          ) : null}
        </h2>
        {hasMore ? (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            loading={loadingMore}
            disabled={loadingMore}
            onClick={handleLoadMore}
            data-ui="comment-load-more"
          >
            {loadingMore ? 'Loading…' : 'Load more comments'}
          </Button>
        ) : null}
      </div>

      {globalError !== null ? (
        <ErrorMessage className="mb-4" data-ui="comment-global-error" role="alert">
          {globalError}
        </ErrorMessage>
      ) : null}

      {canPostRoot ? (
        <Card className="mb-6" data-ui="comment-root-form">
          <form onSubmit={handleCreateRoot} className="flex flex-col gap-3">
            <Field label="Write a comment" error={rootError} invalid={rootError !== null}>
              {(props) => (
                <Textarea
                  {...props}
                  placeholder="Share your thoughts…"
                  rows={3}
                  value={rootContent}
                  onChange={(e) => setRootContent(e.target.value)}
                  disabled={rootSubmitting}
                  data-ui="comment-root-input"
                />
              )}
            </Field>
            <div className="flex items-center justify-end gap-2">
              <Button
                type="submit"
                variant="primary"
                size="sm"
                loading={rootSubmitting}
                disabled={rootSubmitting}
                data-ui="comment-root-submit"
              >
                {rootSubmitting ? 'Posting…' : 'Post comment'}
              </Button>
            </div>
          </form>
        </Card>
      ) : null}

      {threads.length === 0 ? (
        <p className="text-text-muted text-sm" data-ui="comment-empty">
          No comments yet. Be the first to share your thoughts.
        </p>
      ) : (
        <ul className="flex flex-col gap-4" data-ui="comment-list">
          {threads.map((thread) => (
            <li key={thread.comment.id} data-ui="comment-thread">
              <CommentCard
                comment={thread.comment}
                depth={0}
                currentUser={currentUser}
                editingId={editingId}
                editContent={editContent}
                editError={editError}
                editSubmitting={editSubmitting}
                replyingToId={replyingToId}
                replyContent={replyContent}
                replyError={replyError}
                replySubmitting={replySubmitting}
                onStartReply={startReply}
                onCancelReply={cancelReply}
                onSubmitReply={handleReply}
                onReplyChange={setReplyContent}
                onStartEdit={startEdit}
                onCancelEdit={cancelEdit}
                onSubmitEdit={handleUpdate}
                onEditChange={setEditContent}
                onDelete={handleDelete}
              />
              {thread.replies.length > 0 ? (
                <ul className="border-border-subtle mt-3 ml-4 flex flex-col gap-3 border-l pl-4">
                  {thread.replies.map((reply) => (
                    <li key={reply.comment.id} data-ui="comment-reply">
                      <CommentCard
                        comment={reply.comment}
                        depth={1}
                        currentUser={currentUser}
                        editingId={editingId}
                        editContent={editContent}
                        editError={editError}
                        editSubmitting={editSubmitting}
                        replyingToId={replyingToId}
                        replyContent={replyContent}
                        replyError={replyError}
                        replySubmitting={replySubmitting}
                        onStartReply={startReply}
                        onCancelReply={cancelReply}
                        onSubmitReply={handleReply}
                        onReplyChange={setReplyContent}
                        onStartEdit={startEdit}
                        onCancelEdit={cancelEdit}
                        onSubmitEdit={handleUpdate}
                        onEditChange={setEditContent}
                        onDelete={handleDelete}
                      />
                      {reply.replies.length > 0 ? (
                        <ul className="border-border-subtle mt-3 ml-4 flex flex-col gap-3 border-l pl-4">
                          {reply.replies.map((nested) => (
                            <li key={nested.id} data-ui="comment-nested">
                              <CommentCard
                                comment={nested}
                                depth={2}
                                currentUser={currentUser}
                                editingId={editingId}
                                editContent={editContent}
                                editError={editError}
                                editSubmitting={editSubmitting}
                                replyingToId={replyingToId}
                                replyContent={replyContent}
                                replyError={replyError}
                                replySubmitting={replySubmitting}
                                onStartReply={startReply}
                                onCancelReply={cancelReply}
                                onSubmitReply={handleReply}
                                onReplyChange={setReplyContent}
                                onStartEdit={startEdit}
                                onCancelEdit={cancelEdit}
                                onSubmitEdit={handleUpdate}
                                onEditChange={setEditContent}
                                onDelete={handleDelete}
                              />
                            </li>
                          ))}
                        </ul>
                      ) : null}
                    </li>
                  ))}
                </ul>
              ) : null}
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}

interface CommentCardProps {
  comment: CommentDto
  depth: 0 | 1 | 2
  currentUser: UserDto
  editingId: number | null
  editContent: string
  editError: string | null
  editSubmitting: boolean
  replyingToId: number | null
  replyContent: string
  replyError: string | null
  replySubmitting: boolean
  onStartReply: (comment: CommentDto) => void
  onCancelReply: () => void
  onSubmitReply: (parentCommentId: number) => void
  onReplyChange: (value: string) => void
  onStartEdit: (comment: CommentDto) => void
  onCancelEdit: () => void
  onSubmitEdit: (commentId: number) => void
  onEditChange: (value: string) => void
  onDelete: (commentId: number) => void
}

function CommentCard({
  comment,
  depth,
  currentUser,
  editingId,
  editContent,
  editError,
  editSubmitting,
  replyingToId,
  replyContent,
  replyError,
  replySubmitting,
  onStartReply,
  onCancelReply,
  onSubmitReply,
  onReplyChange,
  onStartEdit,
  onCancelEdit,
  onSubmitEdit,
  onEditChange,
  onDelete,
}: CommentCardProps) {
  const isOwner = comment.user_id === currentUser.id
  const isEditing = editingId === comment.id
  const isReplying = replyingToId === comment.id
  const canReply = depth < 2

  return (
    <Card data-ui="comment-card" data-comment-id={comment.id} data-comment-depth={depth}>
      {isEditing ? (
        <div className="flex flex-col gap-3">
          <Field label="Edit comment" error={editError} invalid={editError !== null}>
            {(props) => (
              <Textarea
                {...props}
                rows={3}
                value={editContent}
                onChange={(e) => onEditChange(e.target.value)}
                disabled={editSubmitting}
                data-ui="comment-edit-input"
              />
            )}
          </Field>
          <div className="flex items-center justify-end gap-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              disabled={editSubmitting}
              onClick={onCancelEdit}
              data-ui="comment-edit-cancel"
            >
              Cancel
            </Button>
            <Button
              type="button"
              variant="primary"
              size="sm"
              loading={editSubmitting}
              disabled={editSubmitting}
              onClick={() => onSubmitEdit(comment.id)}
              data-ui="comment-edit-submit"
            >
              {editSubmitting ? 'Saving…' : 'Save'}
            </Button>
          </div>
        </div>
      ) : (
        <>
          <div className="mb-2 flex items-center justify-between gap-2">
            <div className="flex items-center gap-2 text-xs">
              <span className="text-text font-medium" data-ui="comment-author">
                {isOwner ? 'You' : shortUserId(comment.user_id)}
              </span>
              <time className="text-text-subtle" dateTime={comment.created_at}>
                {formatAbsolute(comment.created_at)}
              </time>
            </div>
            <div className="flex items-center gap-1">
              {canReply ? (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => onStartReply(comment)}
                  data-ui="comment-reply-button"
                >
                  Reply
                </Button>
              ) : null}
              {isOwner ? (
                <>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => onStartEdit(comment)}
                    data-ui="comment-edit-button"
                  >
                    Edit
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => onDelete(comment.id)}
                    data-ui="comment-delete-button"
                  >
                    Delete
                  </Button>
                </>
              ) : null}
            </div>
          </div>
          <p
            className="text-text whitespace-pre-wrap text-sm leading-relaxed"
            data-ui="comment-content"
          >
            {comment.content}
          </p>
        </>
      )}

      {isReplying ? (
        <div className="mt-3 flex flex-col gap-3">
          <Field label="Write a reply" error={replyError} invalid={replyError !== null}>
            {(props) => (
              <Textarea
                {...props}
                placeholder="Write a reply…"
                rows={3}
                value={replyContent}
                onChange={(e) => onReplyChange(e.target.value)}
                disabled={replySubmitting}
                data-ui="comment-reply-input"
              />
            )}
          </Field>
          <div className="flex items-center justify-end gap-2">
            <Button
              type="button"
              variant="ghost"
              size="sm"
              disabled={replySubmitting}
              onClick={onCancelReply}
              data-ui="comment-reply-cancel"
            >
              Cancel
            </Button>
            <Button
              type="button"
              variant="primary"
              size="sm"
              loading={replySubmitting}
              disabled={replySubmitting}
              onClick={() => onSubmitReply(comment.id)}
              data-ui="comment-reply-submit"
            >
              {replySubmitting ? 'Posting…' : 'Post reply'}
            </Button>
          </div>
        </div>
      ) : null}
    </Card>
  )
}

function shortUserId(userId: string): string {
  return userId.length > 12 ? `${userId.slice(0, 8)}…` : userId
}

function formatAbsolute(iso: string): string {
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
