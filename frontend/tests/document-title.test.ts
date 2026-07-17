import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import {
  buildDocumentTitle,
  staticRoutePageTitle,
  titleDisplayWidth,
} from '../src/utils/documentTitle.ts'

test('short page titles include the displayed site title', () => {
  assert.equal(
    buildDocumentTitle('知识库', 'Mark-Share-X_用AI学AI'),
    '知识库 - MarkShareX_用AI学AI',
  )
})

test('long article titles drop the site suffix before truncating', () => {
  assert.equal(
    buildDocumentTitle(
      'Rust 错误处理深度解构：Result 与 ? 操作符',
      'Mark-Share-X_用AI学AI',
    ),
    'Rust 错误处理深度解构：Result 与 ? 操作符',
  )
})

test('overlong page titles are truncated by mixed-language display width', () => {
  const title = buildDocumentTitle(
    '深入理解 Rust 所有权生命周期错误处理异步编程与生产环境最佳实践完整指南',
    'Mark-Share-X_用AI学AI',
  )

  assert.equal(title.endsWith('…'), true)
  assert.equal(title.includes('MarkShareX'), false)
  assert.equal(titleDisplayWidth(title) <= 60, true)
})

test('static route titles cover public SPA routes without claiming dynamic routes', () => {
  assert.equal(staticRoutePageTitle('home'), '')
  assert.equal(staticRoutePageTitle('knowledge-base'), '知识库')
  assert.equal(staticRoutePageTitle('search'), '搜索结果')
  assert.equal(staticRoutePageTitle('login'), '登录')
  assert.equal(staticRoutePageTitle('changelog'), '更新日志')
  assert.equal(staticRoutePageTitle('post'), undefined)
  assert.equal(staticRoutePageTitle('category'), undefined)
})

test('static and dynamic front-end routes each have a document title owner', () => {
  const source = (relativePath: string) => readFileSync(
    new URL(`../src/${relativePath}`, import.meta.url),
    'utf8',
  )

  assert.match(source('App.vue'), /useDocumentTitle\(staticPageTitle\)/)
  const postDetail = source('views/front/PostDetail.vue')
  assert.match(postDetail, /useDocumentTitle\(postDocumentTitle\)/)
  assert.match(postDetail, /const requestId = \+\+postRequestId/)
  assert.match(postDetail, /requestId !== postRequestId \|\| currentSlug\(\) !== slug/)
  assert.match(postDetail, /postTitleState\.value = 'loading'/)
  assert.match(postDetail, /postTitleState\.value = 'not-found'/)
  assert.match(postDetail, /onMounted\(\(\) => \{\s*loadPost\(\)\s*\}\)/)
  assert.match(postDetail, /let commentsRequestId = 0/)
  assert.match(postDetail, /requestId !== commentsRequestId \|\| post\.value\?\.id !== postId/)
  assert.match(postDetail, /let likeActionRequestId = 0/)
  assert.match(postDetail, /actionId !== likeActionRequestId/)
  assert.match(postDetail, /requestId !== postRequestId/)
  assert.match(postDetail, /post\.value\?\.id !== postId/)
  assert.match(source('views/front/CategoryPosts.vue'), /useDocumentTitle\(categoryName\)/)
  assert.match(source('views/front/TagPosts.vue'), /useDocumentTitle\(tagName\)/)
  assert.match(source('views/front/AuthorPosts.vue'), /useDocumentTitle\(authorName\)/)
  assert.match(source('views/front/ArticleFilter.vue'), /useDocumentTitle\(documentPageTitle\)/)
})
