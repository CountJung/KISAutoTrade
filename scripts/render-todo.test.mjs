import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { chromium } from '@playwright/test';
import { parseTasks, inlineMarkdown, renderTodo, checkFreshness } from './render-todo.mjs';

const source = readFileSync(new URL('../docs/todo.md', import.meta.url), 'utf8');
const htmlPath = new URL('../docs/todo.html', import.meta.url);

test('open task coverage excludes completed history and nested subcriteria', () => {
  const fixture = `## 잔여\n- [x] 완료 이력\n  - [ ] 과거 하위 항목\n- [ ] **🟢 자동 구현** **SOXQ-P2-01 — 구현**\n  - [ ] 하위 조건\n  - [x] 구현 일부\n\n## 보류\n- [ ] **🟠 사용자 판단 대기** 기존 보류 해제\n  - 근거를 보존한다.\n- [ ] **🟠 사용자 검증·자료 대기** 원본 자료\n`;
  const tasks = parseTasks(fixture);
  assert.equal(tasks.length, 3);
  assert.deepEqual(tasks.map((task) => task.status), ['implementation', 'decision', 'validation']);
  assert.equal(tasks[0].title, '**SOXQ-P2-01 — 구현**');
  assert.equal(tasks[0].body.length, 2);
  assert.equal(tasks[1].section, '보류');
  assert.throws(() => parseTasks('- [ ] 상태 없이 추가한 항목'), /상태를 명시/u);
  assert.throws(() => parseTasks(fixture + '- [ ] **🟢 자동 구현** **SOXQ-P2-01 — 중복**'), /중복/u);
});

test('source changes invalidate HTML and markup cannot execute source HTML or unsafe links', () => {
  const html = renderTodo(source);
  checkFreshness(source, html);
  assert.throws(() => checkFreshness(source + '\n새로운 설명\n', html), /원본과 다릅니다/u);
  const escaped = inlineMarkdown('<script>bad()</script> [unsafe](javascript:bad) [good](research/report.md) `**literal**`');
  assert.ok(!escaped.includes('<script>'));
  assert.ok(!escaped.includes('href="javascript:'));
  assert.ok(escaped.includes('href="research/report.md"'));
  assert.ok(escaped.includes('<code>**literal**</code>'));
});

test('generated document preserves every open source item and supports keyboard filters at mobile width', async () => {
  const browser = await chromium.launch({ headless: true });
  try {
    const page = await browser.newPage({ viewport: { width: 375, height: 812 } });
    const errors = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.goto(htmlPath.href);
    const tasks = parseTasks(source);
    assert.equal(await page.locator('article.task').count(), tasks.length);
    const visibleTitles = tasks.map((task) => task.title.replace(/\[([^\]]+)\]\([^)]+\)/gu, '$1').replace(/\*\*/gu, '').replace(/`/gu, ''));
    for (const [index, task] of tasks.entries()) {
      const card = page.locator(`#${task.id}`);
      assert.equal(await card.getAttribute('data-source-line'), String(task.line));
      const visibleTitle = visibleTitles[index];
      assert.ok((await card.textContent()).includes(visibleTitle), task.id);
      for (const link of task.title.matchAll(/\[([^\]]+)\]\(([^)]+)\)/gu)) {
        assert.equal(await card.getByRole('link', { name: link[1], exact: true }).getAttribute('href'), link[2]);
      }
    }
    const waitingCount = tasks.filter((task) => task.status !== 'implementation').length;
    await page.getByLabel('상태별 보기').selectOption('waiting');
    assert.equal(await page.locator('article.task:visible').count(), waitingCount);
    assert.equal(await page.locator('article.implementation:visible').count(), 0);
    const badge = page.locator('article.decision .badge').first();
    const colors = await badge.evaluate((element) => ({ text: getComputedStyle(element).color, background: getComputedStyle(element).backgroundColor }));
    assert.notEqual(colors.text, colors.background);
    await page.getByLabel('상태별 보기').selectOption('all');
    await page.getByLabel('할일 검색').focus();
    // Search an actual open title: completing a task must not break this UI test.
    assert.ok(visibleTitles.length > 0);
    await page.keyboard.type(visibleTitles[0]);
    assert.equal(await page.locator('article.task:visible').count(), 1);
    await page.getByLabel('할일 검색').fill('없는항목_결과없음');
    assert.equal(await page.locator('article.task:visible').count(), 0);
    assert.equal(await page.locator('#empty').isVisible(), true);
    await page.getByLabel('할일 검색').fill('');
    assert.equal(await page.locator('article.task:visible').count(), tasks.length);
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
    assert.deepEqual(errors, []);
    await page.emulateMedia({ colorScheme: 'dark' });
    assert.equal(await page.getByRole('heading', { name: 'KISAutoTrade — 열린 할일' }).isVisible(), true);
    // No script is needed to read the complete document and evidence links offline.
    const noScript = await browser.newContext({ javaScriptEnabled: false });
    const plain = await noScript.newPage();
    await plain.goto(htmlPath.href);
    assert.equal(await plain.locator('article.task:visible').count(), tasks.length);
    await noScript.close();
  } finally {
    await browser.close();
  }
});
