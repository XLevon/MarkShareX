import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick, reactive } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import ArticleFilter from '@/views/front/ArticleFilter.vue'

let route: { path: string; params: { code?: string } }
const mocks = vi.hoisted(() => ({
  routerPush: vi.fn(),
  fetchPosts: vi.fn(),
  fetchArticleTypes: vi.fn(),
  fetchArticleStatuses: vi.fn(),
}))

vi.mock('vue-router', () => ({
  useRoute: () => route,
  useRouter: () => ({ push: mocks.routerPush }),
}))
vi.mock('@/api/posts', () => ({ fetchPosts: mocks.fetchPosts }))
vi.mock('@/api/admin', () => ({
  fetchArticleTypes: mocks.fetchArticleTypes,
  fetchArticleStatuses: mocks.fetchArticleStatuses,
}))
vi.mock('@/composables/useDocumentTitle', () => ({ useDocumentTitle: vi.fn() }))

const fetchPosts = mocks.fetchPosts
const fetchArticleTypes = mocks.fetchArticleTypes
const fetchArticleStatuses = mocks.fetchArticleStatuses

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (reason?: unknown) => void
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  let reject!: (reason?: unknown) => void
  const promise = new Promise<T>((res, rej) => {
    resolve = res
    reject = rej
  })
  return { promise, resolve, reject }
}

function metaResponse(label: string, code = label.toLowerCase()) {
  return {
    data: {
      data: [{ code, display_name: label, color: '#123456', post_count: 1 }],
    },
  }
}

interface TestPost { id: number; title: string }
function postsResponse(posts: TestPost[], total = posts.length) {
  return {
    data: {
      data: posts,
      pagination: { total },
    },
  }
}
function onePost(id: number, title: string, total = 1) {
  return postsResponse([{ id, title }], total)
}

const observerInstances: MockIntersectionObserver[] = []
class MockIntersectionObserver {
  callback: IntersectionObserverCallback
  disconnected = false
  observed: Element[] = []
  disconnectCount = 0

  constructor(callback: IntersectionObserverCallback) {
    this.callback = callback
    observerInstances.push(this)
  }

  observe(element: Element) {
    this.observed.push(element)
  }

  disconnect() {
    this.disconnected = true
    this.disconnectCount += 1
  }

  trigger(isIntersecting = true) {
    this.callback([{ isIntersecting } as IntersectionObserverEntry], this as unknown as IntersectionObserver)
  }
}

const wrappers: VueWrapper[] = []
function trackedMount(options: Parameters<typeof mount>[1]) {
  const wrapper = mount(ArticleFilter, options)
  wrappers.push(wrapper)
  return wrapper
}

function mountFilter(code = 'alpha', filterType: 'type' | 'status' = 'type') {
  const prefix = filterType === 'type' ? 'type' : 'status'
  route = reactive({ path: `/${prefix}/${code}`, params: { code } })
  return trackedMount({
    props: { filterType },
    global: {
      plugins: [createPinia()],
      stubs: {
        RouterLink: { template: '<a><slot /></a>' },
        PostCard: {
          props: ['post'],
          template: '<article class="post-card-stub" :data-post-id="post.id">{{ post.title }}</article>',
        },
      },
    },
  })
}

async function changeRoute(code: string, filterType: 'type' | 'status' = 'type') {
  const prefix = filterType === 'type' ? 'type' : 'status'
  route.path = `/${prefix}/${code}`
  route.params.code = code
  await nextTick()
  await flushPromises()
}

describe('ArticleFilter latest-route request ownership', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    observerInstances.length = 0
    vi.stubGlobal('IntersectionObserver', MockIntersectionObserver)
    fetchArticleTypes.mockResolvedValue(metaResponse('TYPE'))
    fetchArticleStatuses.mockResolvedValue(metaResponse('STATUS'))
  })

  afterEach(() => {
    while (wrappers.length) {
      const wrapper = wrappers.pop()
      if (wrapper?.exists()) wrapper.unmount()
    }
    vi.unstubAllGlobals()
  })

  it('keeps loading for the newest route and ignores an older first-page response', async () => {
    const alpha = deferred<ReturnType<typeof onePost>>()
    const beta = deferred<ReturnType<typeof onePost>>()
    fetchPosts.mockReturnValueOnce(alpha.promise).mockReturnValueOnce(beta.promise)

    const wrapper = mountFilter('alpha')
    await flushPromises()
    await changeRoute('beta')

    alpha.resolve(onePost(1, 'OLD ALPHA'))
    await flushPromises()
    expect(wrapper.text()).toContain('加载中...')
    expect(wrapper.text()).not.toContain('OLD ALPHA')

    beta.resolve(onePost(2, 'NEW BETA'))
    await flushPromises()
    expect(wrapper.text()).toContain('NEW BETA')
    expect(wrapper.text()).not.toContain('OLD ALPHA')
    expect(observerInstances.filter(instance => !instance.disconnected)).toHaveLength(1)
  })

  it('does not append an old filter load-more response to the new route', async () => {
    const oldMore = deferred<ReturnType<typeof onePost>>()
    const beta = deferred<ReturnType<typeof onePost>>()
    fetchPosts
      .mockResolvedValueOnce(onePost(1, 'ALPHA FIRST', 2))
      .mockReturnValueOnce(oldMore.promise)
      .mockReturnValueOnce(beta.promise)

    const wrapper = mountFilter('alpha')
    await flushPromises()
    observerInstances[0].trigger()
    await flushPromises()
    await changeRoute('beta')
    beta.resolve(onePost(3, 'BETA FIRST'))
    await flushPromises()

    oldMore.resolve(onePost(2, 'STALE ALPHA PAGE'))
    await flushPromises()
    expect(wrapper.text()).toContain('BETA FIRST')
    expect(wrapper.text()).not.toContain('ALPHA FIRST')
    expect(wrapper.text()).not.toContain('STALE ALPHA PAGE')
  })

  it('uses one page size for the entire generation so page-number offsets neither overlap nor skip', async () => {
    const allPosts = Array.from({ length: 8 }, (_, index) => ({
      id: index + 1,
      title: `POST ${index + 1}`,
    }))
    fetchPosts.mockImplementation((params: { page: number; page_size: number }) => {
      const offset = (params.page - 1) * params.page_size
      return Promise.resolve(postsResponse(allPosts.slice(offset, offset + params.page_size), allPosts.length))
    })

    const wrapper = mountFilter('alpha')
    await flushPromises()
    observerInstances[0].trigger()
    await flushPromises()

    expect(fetchPosts.mock.calls.map(call => call[0])).toEqual([
      expect.objectContaining({ page: 1, page_size: 5, article_type: 'alpha' }),
      expect.objectContaining({ page: 2, page_size: 5, article_type: 'alpha' }),
    ])
    const rendered = wrapper.findAll('.post-card-stub').map(card => card.text())
    expect(rendered).toEqual(allPosts.map(post => post.title))
    expect(new Set(rendered).size).toBe(8)
  })

  it('retries the same next page with the same page size after a load-more failure', async () => {
    fetchPosts
      .mockResolvedValueOnce(postsResponse(
        Array.from({ length: 5 }, (_, index) => ({ id: index + 1, title: `FIRST ${index + 1}` })),
        8,
      ))
      .mockRejectedValueOnce(new Error('temporary page failure'))
      .mockResolvedValueOnce(postsResponse([
        { id: 6, title: 'SECOND 6' },
        { id: 7, title: 'SECOND 7' },
        { id: 8, title: 'SECOND 8' },
      ], 8))

    const wrapper = mountFilter('alpha')
    await flushPromises()
    observerInstances[0].trigger()
    await flushPromises()
    observerInstances[0].trigger()
    await flushPromises()

    expect(fetchPosts.mock.calls[1][0]).toMatchObject({ page: 2, page_size: 5, article_type: 'alpha' })
    expect(fetchPosts.mock.calls[2][0]).toMatchObject({ page: 2, page_size: 5, article_type: 'alpha' })
    expect(wrapper.findAll('.post-card-stub')).toHaveLength(8)
  })

  it('isolates type metadata and post params from a realistic type-to-status prop transition', async () => {
    const oldTypeMeta = deferred<ReturnType<typeof metaResponse>>()
    fetchArticleTypes.mockReturnValueOnce(oldTypeMeta.promise)
    fetchArticleStatuses.mockResolvedValueOnce(metaResponse('CURRENT STATUS', 'current'))
    fetchPosts.mockResolvedValueOnce(onePost(20, 'STATUS POST'))

    const wrapper = mountFilter('alpha', 'type')
    await flushPromises()
    route.path = '/status/current'
    route.params.code = 'current'
    await wrapper.setProps({ filterType: 'status' })
    await flushPromises()

    oldTypeMeta.resolve(metaResponse('STALE TYPE', 'alpha'))
    await flushPromises()

    expect(fetchArticleStatuses).toHaveBeenCalledTimes(1)
    expect(fetchPosts).toHaveBeenCalledTimes(1)
    expect(fetchPosts.mock.calls[0][0]).toMatchObject({
      page: 1,
      page_size: 5,
      article_status: 'current',
    })
    expect(fetchPosts.mock.calls[0][0]).not.toHaveProperty('article_type')
    expect(wrapper.text()).toContain('CURRENT STATUS')
    expect(wrapper.text()).toContain('STATUS POST')
    expect(wrapper.text()).not.toContain('STALE TYPE')
  })

  it('does not let an old load-more finally clear the new route active loading indicator', async () => {
    const oldMore = deferred<ReturnType<typeof onePost>>()
    const newMore = deferred<ReturnType<typeof onePost>>()
    fetchPosts
      .mockResolvedValueOnce(onePost(1, 'ALPHA FIRST', 2))
      .mockReturnValueOnce(oldMore.promise)
      .mockResolvedValueOnce(onePost(2, 'BETA FIRST', 2))
      .mockReturnValueOnce(newMore.promise)

    const wrapper = mountFilter('alpha')
    await flushPromises()
    observerInstances[0].trigger()
    await flushPromises()
    await changeRoute('beta')
    observerInstances[1].trigger()
    await nextTick()
    expect(wrapper.find('.spinner').exists()).toBe(true)

    oldMore.resolve(onePost(10, 'OLD MORE'))
    await flushPromises()
    expect(wrapper.find('.spinner').exists()).toBe(true)

    newMore.resolve(onePost(3, 'BETA SECOND', 2))
    await flushPromises()
    expect(wrapper.find('.spinner').exists()).toBe(false)
    expect(wrapper.text()).not.toContain('OLD MORE')
  })

  it('disconnects an active observer and rejects a late load-more commit after unmount', async () => {
    const lateMore = deferred<ReturnType<typeof onePost>>()
    fetchPosts
      .mockResolvedValueOnce(onePost(1, 'BEFORE UNMOUNT', 2))
      .mockReturnValueOnce(lateMore.promise)
    const wrapper = mountFilter('alpha')
    await flushPromises()
    const vm = wrapper.vm as unknown as { posts: TestPost[] }
    observerInstances[0].trigger()
    await nextTick()

    wrapper.unmount()
    expect(observerInstances[0].disconnected).toBe(true)
    expect(observerInstances[0].disconnectCount).toBeGreaterThanOrEqual(1)
    lateMore.resolve(onePost(2, 'AFTER UNMOUNT', 2))
    await flushPromises()

    expect(vm.posts.map(post => post.title)).toEqual(['BEFORE UNMOUNT'])
  })

  it('rejects late metadata after unmount', async () => {
    const lateMeta = deferred<ReturnType<typeof metaResponse>>()
    fetchArticleTypes.mockReturnValueOnce(lateMeta.promise)
    route = reactive({ path: '/types', params: {} })
    const wrapper = trackedMount({
      props: { filterType: 'type' },
      global: {
        plugins: [createPinia()],
        stubs: { RouterLink: { template: '<a><slot /></a>' } },
      },
    })
    await flushPromises()
    const vm = wrapper.vm as unknown as { metaList: unknown[] }

    wrapper.unmount()
    lateMeta.resolve(metaResponse('AFTER UNMOUNT'))
    await flushPromises()

    expect(vm.metaList).toEqual([])
    expect(observerInstances).toHaveLength(0)
  })

  it('ignores disconnected observer callbacks and keeps exactly one live observer across routes', async () => {
    fetchPosts
      .mockResolvedValueOnce(onePost(1, 'ALPHA FIRST', 2))
      .mockResolvedValueOnce(onePost(2, 'BETA FIRST', 2))
      .mockResolvedValueOnce(onePost(3, 'BETA SECOND', 2))
      .mockResolvedValueOnce(onePost(4, 'GAMMA FIRST', 1))

    mountFilter('alpha')
    await flushPromises()
    const alphaObserver = observerInstances[0]
    expect(alphaObserver.observed).toHaveLength(1)

    await changeRoute('beta')
    const betaObserver = observerInstances[1]
    expect(alphaObserver.disconnected).toBe(true)
    alphaObserver.trigger()
    await flushPromises()
    expect(fetchPosts).toHaveBeenCalledTimes(2)

    betaObserver.trigger()
    await flushPromises()
    expect(fetchPosts).toHaveBeenCalledTimes(3)
    await changeRoute('gamma')

    expect(betaObserver.disconnected).toBe(true)
    expect(observerInstances[2].observed).toHaveLength(1)
    expect(observerInstances.filter(instance => !instance.disconnected)).toHaveLength(1)
    betaObserver.trigger()
    await flushPromises()
    expect(fetchPosts).toHaveBeenCalledTimes(4)
  })
})
