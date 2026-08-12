import { expect, test, type Page } from '@playwright/test';

const email = process.env.JURY_E2E_EMAIL;
const password = process.env.JURY_E2E_PASSWORD;

test.skip(!email || !password, 'Set JURY_E2E_EMAIL and JURY_E2E_PASSWORD for an isolated test account.');

async function signIn(page: Page) {
  await page.addInitScript(() => localStorage.setItem('jury-assistant-locale', 'en'));
  await page.goto('/');
  await page.locator('[data-hydrated="true"]').waitFor();
  await page.getByLabel('Email').fill(email!);
  await page.getByLabel('Password').fill(password!);
  const loginResponse = page.waitForResponse((response) => response.url().endsWith('/auth/login') && response.request().method() === 'POST');
  await page.getByRole('button', { name: 'Sign in' }).click();
  expect((await loginResponse).status()).toBe(200);
}

test('signs in, loads the dashboard, and signs out', async ({ page }) => {
  await signIn(page);

  await expect(page.getByRole('button', { name: 'Sign out' })).toBeVisible();
  await expect(page.getByText('Overview', { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Sign out' }).click();
  await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible();
});

test('keeps the dashboard within desktop and mobile viewports', async ({ page }) => {
  for (const [index, viewport] of [{ width: 1440, height: 900 }, { width: 390, height: 844 }].entries()) {
    if (index > 0) await page.evaluate(() => localStorage.clear());
    await page.setViewportSize(viewport);
    await signIn(page);
    await expect(page.getByRole('heading', { name: 'Jury Assistant' })).toBeVisible();
    const hasHorizontalOverflow = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
    expect(hasHorizontalOverflow).toBe(false);
  }
});
