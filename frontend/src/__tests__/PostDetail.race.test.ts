import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import PostDetail from '@/views/front/PostDetail.vue'

let route: { params: { slug: string }; fullPath: string }
const mocks = vi.hoisted(() => ({
  apiGet: vi.fn(),
  apiPost: vi.fn(),
  fetchComments: vi.fn(),
  createComment: vi.fn(),
  recordReadLog: vi.fn().mockResolvedValue(undefined),
  routerReplace: vi.fn(),
  routerPush: vi.fn(),
  authenticated: true,
  authUser: { id: 99, role: 'admin' } as { id: number; role: string } | null,
}))

vi.mock('vue-router', () => ({
  useRoute: () => route,
  useRouter: () => ({
    replace: mocks.routerReplace,
    push: mocks.routerPush,
    back: vi.fn(),
  }),
}))
vi.mock('@/api/index', () => ({
  default: { get: mocks.apiGet, post: mocks.apiPost },
}))
vi.mock('@/api/admin', () => ({ recordReadLog: mocks.recordReadLog }))
vi.mock('@/api/comments', () => ({
  fetchComments: mocks.fetchComments,
  createComment: mocks.createComment,
}))
vi.mock('@/stores/auth', () => ({
  useAuthStore: () => ({
    get isAuthenticated() { return mocks.authenticated },
    get user() { return mocks.authUser },
  }),
}))
vi.mock('@/stores/settings', () => ({
  useSettingsStore: () => ({ settings: { guest_copy_enabled: 'true' } }),
}))
vi.mock('@/composables/useDocumentTitle', () => ({ useDocumentTitle: vi.fn() }))

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
}
function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  const promise = new Promise<T>(res => { resolve = res })
  return { promise, resolve }
}

function body<T>(data: T) {
  return { data: { data } }
}
function post(id: number, slug: string, title: string, likeCount: number) {
  return {
    id,
    slug,
    title,
    summary: title,
    content_html: '',
    like_count: likeCount,
    view_count: 1,
    allow_comment: true,
    tags: [],
  }
}
function comment(id: number, label: string) {
  return {
    id,
    post_id: id,
    user_id: null,
    parent_id: null,
    author_name: label,
    content: label,
    content_html: label,
    status: 'approved',
    like_count: 0,
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
    replies: [],
  }
}

const wrappers: VueWrapper[] = []
function mountDetail(slug = 'alpha') {
  route = reactive({ params: { slug }, fullPath: `/post/${slug}` })
  const wrapper = mount(PostDetail, {
    global: {
      stubs: {
        RouterLink: { template: '<a><slot /></a>' },
        Teleport: true,
        CodeCopyWrapper: { template: '<div class="content-stub"></div>' },
        ActionBar: {
          props: ['prev', 'next', 'likeCount', 'liked', 'likeLoading'],
          emits: ['toggle-like'],
          template: `
            <div class="action-bar-stub">
              <span class="adjacent-stub">{{ prev?.title || '' }}|{{ next?.title || '' }}</span>
              <span class="like-stub">{{ likeCount }}|{{ liked }}|{{ likeLoading }}</span>
              <button class="toggle-like" @click="$emit('toggle-like')">like</button>
            </div>
          `,
        },
      },
    },
  })
  wrappers.push(wrapper)
  return wrapper
}

async function changeSlug(slug: string) {
  route.params.slug = slug
  route.fullPath = `/post/${slug}`
  await nextTick()
  await flushPromises()
}

describe('PostDetail latest-post request ownership', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.authenticated = true
    mocks.authUser = { id: 99, role: 'admin' }
    mocks.recordReadLog.mockResolvedValue(undefined)
    vi.stubGlobal('IntersectionObserver', class {
      observe() {}
      disconnect() {}
    })
  })

  afterEach(() => {
    while (wrappers.length) {
      const wrapper = wrappers.pop()
      if (wrapper?.exists()) wrapper.unmount()
    }
    vi.unstubAllGlobals()
  })

  it('keeps the newest primary post and loading owner when an older slug resolves late', async () => {
    const alphaPost = deferred<ReturnType<typeof body>>()
    const betaPost = deferred<ReturnType<typeof body>>()
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return alphaPost.promise
      if (url === '/posts/slug/beta') return betaPost.promise
      if (url === '/posts/2/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/2/like-status') return Promise.resolve(body({ liked: true, like_count: 20 }))
      throw new Error(`unexpected GET ${url}`)
    })
    mocks.fetchComments.mockResolvedValue(body([]))

    const wrapper = mountDetail('alpha')
    await flushPromises()
    expect(wrapper.text()).toContain('加载中...')
    await changeSlug('beta')
    expect(wrapper.text()).toContain('加载中...')

    alphaPost.resolve(body(post(1, 'alpha', 'STALE ALPHA POST', 10)))
    await flushPromises()
    expect(wrapper.text()).toContain('加载中...')
    expect(wrapper.text()).not.toContain('STALE ALPHA POST')
    expect(mocks.recordReadLog).not.toHaveBeenCalled()

    betaPost.resolve(body(post(2, 'beta', 'BETA POST', 20)))
    await flushPromises()
    expect(wrapper.text()).toContain('BETA POST')
    expect(wrapper.text()).not.toContain('加载中...')
    expect(mocks.recordReadLog).toHaveBeenCalledWith(expect.objectContaining({ post_id: 2 }))
  })

  it('ignores a primary post response that resolves after unmount', async () => {
    const alphaPost = deferred<ReturnType<typeof body>>()
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return alphaPost.promise
      throw new Error(`unexpected GET ${url}`)
    })
    mocks.fetchComments.mockResolvedValue(body([]))

    const wrapper = mountDetail('alpha')
    await flushPromises()
    wrapper.unmount()
    alphaPost.resolve(body(post(1, 'alpha', 'UNMOUNTED ALPHA POST', 10)))
    await flushPromises()

    expect(mocks.recordReadLog).not.toHaveBeenCalled()
    expect(mocks.fetchComments).not.toHaveBeenCalled()
    expect(mocks.apiGet).toHaveBeenCalledTimes(1)
  })

  it('does not let a comment mutation from the previous article clear or reload the new article', async () => {
    const alphaCreate = deferred<ReturnType<typeof body>>()
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return Promise.resolve(body(post(1, 'alpha', 'ALPHA POST', 10)))
      if (url === '/posts/1/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/1/like-status') return Promise.resolve(body({ liked: false, like_count: 10 }))
      if (url === '/posts/slug/beta') return Promise.resolve(body(post(2, 'beta', 'BETA POST', 20)))
      if (url === '/posts/2/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/2/like-status') return Promise.resolve(body({ liked: false, like_count: 20 }))
      throw new Error(`unexpected GET ${url}`)
    })
    mocks.fetchComments.mockImplementation((postId: number) => Promise.resolve(body([comment(postId, `${postId} COMMENT`)])))
    mocks.createComment.mockReturnValue(alphaCreate.promise)

    const wrapper = mountDetail('alpha')
    await flushPromises()
    await wrapper.find('textarea[placeholder^="写下你的评论"]').setValue('alpha pending')
    await wrapper.find('form').trigger('submit')
    expect(wrapper.text()).toContain('提交中...')

    await changeSlug('beta')
    const betaForm = wrapper.find('textarea[placeholder^="写下你的评论"]')
    await betaForm.setValue('beta draft')
    expect((betaForm.element as HTMLTextAreaElement).value).toBe('beta draft')
    const callsBeforeAlphaResolve = mocks.fetchComments.mock.calls.length

    alphaCreate.resolve(body(comment(101, 'ALPHA CREATED')))
    await flushPromises()

    expect((wrapper.find('textarea[placeholder^="写下你的评论"]').element as HTMLTextAreaElement).value).toBe('beta draft')
    expect(wrapper.text()).not.toContain('提交中...')
    expect(mocks.fetchComments).toHaveBeenCalledTimes(callsBeforeAlphaResolve)
  })

  it('does not let an old reply mutation close or clear a reply form on the new article', async () => {
    const alphaReply = deferred<ReturnType<typeof body>>()
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return Promise.resolve(body(post(1, 'alpha', 'ALPHA POST', 10)))
      if (url === '/posts/1/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/1/like-status') return Promise.resolve(body({ liked: false, like_count: 10 }))
      if (url === '/posts/slug/beta') return Promise.resolve(body(post(2, 'beta', 'BETA POST', 20)))
      if (url === '/posts/2/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/2/like-status') return Promise.resolve(body({ liked: false, like_count: 20 }))
      throw new Error(`unexpected GET ${url}`)
    })
    mocks.fetchComments.mockImplementation((postId: number) => Promise.resolve(body([comment(postId, `${postId} COMMENT`)])))
    mocks.createComment.mockReturnValue(alphaReply.promise)

    const wrapper = mountDetail('alpha')
    await flushPromises()
    await wrapper.findAll('button').find(button => button.text().trim() === '回复')!.trigger('click')
    await wrapper.find('textarea[placeholder^="回复 @"]').setValue('alpha reply')
    const replyButtons = wrapper.findAll('button').filter(button => button.text().trim() === '回复')
    const alphaSubmit = replyButtons[replyButtons.length - 1]
    await alphaSubmit.trigger('click')

    await changeSlug('beta')
    await wrapper.findAll('button').find(button => button.text().trim() === '回复')!.trigger('click')
    await wrapper.find('textarea[placeholder^="回复 @"]').setValue('beta reply draft')
    const callsBeforeAlphaResolve = mocks.fetchComments.mock.calls.length

    alphaReply.resolve(body(comment(102, 'ALPHA REPLY CREATED')))
    await flushPromises()

    expect((wrapper.find('textarea[placeholder^="回复 @"]').element as HTMLTextAreaElement).value).toBe('beta reply draft')
    expect(mocks.fetchComments).toHaveBeenCalledTimes(callsBeforeAlphaResolve)
  })

  it('does not run comment mutation continuations after unmount', async () => {
    const alphaCreate = deferred<ReturnType<typeof body>>()
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return Promise.resolve(body(post(1, 'alpha', 'ALPHA POST', 10)))
      if (url === '/posts/1/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/1/like-status') return Promise.resolve(body({ liked: false, like_count: 10 }))
      throw new Error(`unexpected GET ${url}`)
    })
    mocks.fetchComments.mockResolvedValue(body([comment(1, 'ALPHA COMMENT')]))
    mocks.createComment.mockReturnValue(alphaCreate.promise)

    const wrapper = mountDetail('alpha')
    await flushPromises()
    await wrapper.find('textarea[placeholder^="写下你的评论"]').setValue('alpha pending')
    await wrapper.find('form').trigger('submit')
    const callsBeforeUnmount = mocks.fetchComments.mock.calls.length
    wrapper.unmount()
    alphaCreate.resolve(body(comment(103, 'UNMOUNTED CREATED')))
    await flushPromises()
    expect(mocks.fetchComments).toHaveBeenCalledTimes(callsBeforeUnmount)
  })

  it('does not run reply mutation continuations after unmount', async () => {
    const alphaReply = deferred<ReturnType<typeof body>>()
    mocks.authenticated = false
    mocks.authUser = null
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return Promise.resolve(body(post(1, 'alpha', 'ALPHA POST', 10)))
      if (url === '/posts/1/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/1/like-status') return Promise.resolve(body({ liked: false, like_count: 10 }))
      throw new Error(`unexpected GET ${url}`)
    })
    const initialComment = {
      ...comment(1, 'ALPHA COMMENT'),
      replies: [] as unknown[],
    }
    const reloadedComment = {
      ...comment(1, 'ALPHA COMMENT'),
      replies: [{
        ...comment(104, 'UNMOUNTED REPLY CREATED'),
        parent_id: 1,
      }],
    }
    mocks.fetchComments
      .mockResolvedValueOnce(body([initialComment]))
      .mockResolvedValue(body([reloadedComment]))
    mocks.createComment.mockReturnValue(alphaReply.promise)

    const wrapper = mountDetail('alpha')
    await flushPromises()
    const scrollIntoView = vi.fn()
    Object.defineProperty(HTMLElement.prototype, 'scrollIntoView', {
      configurable: true,
      value: scrollIntoView,
    })
    const storageWrite = vi.spyOn(Storage.prototype, 'setItem')
    await wrapper.findAll('button').find(button => button.text().trim() === '回复')!.trigger('click')
    await wrapper.find('input[placeholder="昵称"]').setValue('anonymous reviewer')
    await wrapper.find('textarea[placeholder^="回复 @"]').setValue('alpha pending reply')
    const replyButtons = wrapper.findAll('button').filter(button => button.text().trim() === '回复')
    await replyButtons[replyButtons.length - 1].trigger('click')
    const callsBeforeUnmount = mocks.fetchComments.mock.calls.length
    wrapper.unmount()
    const staleScrollTarget = document.createElement('div')
    staleScrollTarget.id = 'comment-104'
    document.body.appendChild(staleScrollTarget)

    alphaReply.resolve(body(comment(104, 'UNMOUNTED REPLY CREATED')))
    await flushPromises()

    expect.soft(mocks.fetchComments).toHaveBeenCalledTimes(callsBeforeUnmount)
    expect.soft(scrollIntoView).not.toHaveBeenCalled()
    expect.soft(storageWrite).not.toHaveBeenCalled()
    storageWrite.mockRestore()
    staleScrollTarget.remove()
    delete (HTMLElement.prototype as { scrollIntoView?: unknown }).scrollIntoView
  })

  it('does not let an old adjacent response overwrite the newer post navigation', async () => {
    const alphaAdjacent = deferred<ReturnType<typeof body>>()
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return Promise.resolve(body(post(1, 'alpha', 'ALPHA POST', 10)))
      if (url === '/posts/1/adjacent') return alphaAdjacent.promise
      if (url === '/posts/slug/beta') return Promise.resolve(body(post(2, 'beta', 'BETA POST', 20)))
      if (url === '/posts/2/adjacent') return Promise.resolve(body({ prev: { id: 3, slug: 'b-prev', title: 'BETA PREV' }, next: null }))
      if (url === '/posts/2/like-status') return Promise.resolve(body({ liked: false, like_count: 20 }))
      throw new Error(`unexpected GET ${url}`)
    })
    mocks.fetchComments.mockResolvedValue(body([comment(2, 'BETA COMMENT')]))

    const wrapper = mountDetail('alpha')
    await flushPromises()
    await changeSlug('beta')
    expect(wrapper.text()).toContain('BETA PREV')

    alphaAdjacent.resolve(body({ prev: { id: 1, slug: 'a-prev', title: 'STALE ALPHA PREV' }, next: null }))
    await flushPromises()

    expect(wrapper.text()).toContain('BETA POST')
    expect(wrapper.text()).toContain('BETA PREV')
    expect(wrapper.text()).not.toContain('STALE ALPHA PREV')
  })

  it('does not let old comments replace comments loaded for the new post', async () => {
    const alphaComments = deferred<ReturnType<typeof body>>()
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return Promise.resolve(body(post(1, 'alpha', 'ALPHA POST', 10)))
      if (url === '/posts/1/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/1/like-status') return Promise.resolve(body({ liked: false, like_count: 10 }))
      if (url === '/posts/slug/beta') return Promise.resolve(body(post(2, 'beta', 'BETA POST', 20)))
      if (url === '/posts/2/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/2/like-status') return Promise.resolve(body({ liked: true, like_count: 20 }))
      throw new Error(`unexpected GET ${url}`)
    })
    mocks.fetchComments.mockImplementation((postId: number) => (
      postId === 1 ? alphaComments.promise : Promise.resolve(body([comment(2, 'BETA COMMENT')]))
    ))

    const wrapper = mountDetail('alpha')
    await flushPromises()
    await changeSlug('beta')
    expect(wrapper.text()).toContain('BETA COMMENT')

    alphaComments.resolve(body([comment(1, 'STALE ALPHA COMMENT')]))
    await flushPromises()

    expect(wrapper.text()).toContain('BETA COMMENT')
    expect(wrapper.text()).not.toContain('STALE ALPHA COMMENT')
  })

  it('does not let an old like-status response overwrite the newer post like state', async () => {
    const alphaLikeStatus = deferred<ReturnType<typeof body>>()
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return Promise.resolve(body(post(1, 'alpha', 'ALPHA POST', 10)))
      if (url === '/posts/1/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/1/like-status') return alphaLikeStatus.promise
      if (url === '/posts/slug/beta') return Promise.resolve(body(post(2, 'beta', 'BETA POST', 20)))
      if (url === '/posts/2/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/2/like-status') return Promise.resolve(body({ liked: true, like_count: 20 }))
      throw new Error(`unexpected GET ${url}`)
    })
    mocks.fetchComments.mockResolvedValue(body([]))

    const wrapper = mountDetail('alpha')
    await flushPromises()
    expect(mocks.apiGet).toHaveBeenCalledWith('/posts/1/like-status')

    await changeSlug('beta')
    expect(wrapper.find('.like-stub').text()).toContain('20|true|false')
    alphaLikeStatus.resolve(body({ liked: false, like_count: 999 }))
    await flushPromises()

    expect(wrapper.text()).toContain('BETA POST')
    expect(wrapper.find('.like-stub').text()).toContain('20|true|false')
    expect(wrapper.text()).not.toContain('999')
  })

  it('does not let an old pending like action overwrite the newer post like state', async () => {
    const alphaLike = deferred<ReturnType<typeof body>>()
    mocks.apiGet.mockImplementation((url: string) => {
      if (url === '/posts/slug/alpha') return Promise.resolve(body(post(1, 'alpha', 'ALPHA POST', 10)))
      if (url === '/posts/1/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/1/like-status') return Promise.resolve(body({ liked: false, like_count: 10 }))
      if (url === '/posts/slug/beta') return Promise.resolve(body(post(2, 'beta', 'BETA POST', 20)))
      if (url === '/posts/2/adjacent') return Promise.resolve(body({ prev: null, next: null }))
      if (url === '/posts/2/like-status') return Promise.resolve(body({ liked: true, like_count: 20 }))
      throw new Error(`unexpected GET ${url}`)
    })
    mocks.apiPost.mockImplementation((url: string) => {
      if (url === '/posts/1/like') return alphaLike.promise
      throw new Error(`unexpected POST ${url}`)
    })
    mocks.fetchComments.mockResolvedValue(body([]))

    const wrapper = mountDetail('alpha')
    await flushPromises()
    await wrapper.find('.toggle-like').trigger('click')
    expect(mocks.apiPost).toHaveBeenCalledWith('/posts/1/like')

    await changeSlug('beta')
    expect(wrapper.find('.like-stub').text()).toContain('20|true|false')
    alphaLike.resolve(body({ liked: true, like_count: 999 }))
    await flushPromises()

    expect(wrapper.text()).toContain('BETA POST')
    expect(wrapper.find('.like-stub').text()).toContain('20|true|false')
    expect(wrapper.text()).not.toContain('999')
  })
})
