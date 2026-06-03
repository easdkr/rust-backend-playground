/**
 * Barrel for the server-function layer.
 *
 * Routes / loaders / components should import from here, not from the
 * individual files. Keeps the per-feature module list stable as the
 * API surface grows.
 */

export {
  loginServerFn,
  getCurrentSessionServerFn,
  clearSessionServerFn,
} from './auth.ts'

export {
  listPostsServerFn,
  getPostServerFn,
  createPostServerFn,
  updatePostServerFn,
} from './posts.ts'
