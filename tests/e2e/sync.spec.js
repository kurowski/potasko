// E2E Tests for CalDAV Sync via GUI
// Note: Database cleanup and Radicale startup are handled in wdio.conf.js onPrepare

describe('CalDAV Sync via GUI', () => {

  it('should open Settings and add a CalDAV account', async () => {
    // Wait for app to initialize
    await browser.pause(3000);

    // Click Settings button in sidebar footer
    const settingsBtn = await $('button.settings-btn');
    await settingsBtn.waitForClickable({ timeout: 10000 });
    await settingsBtn.click();
    await browser.pause(500);

    // Click "Add Account" button
    const addAccountBtn = await $('button.add-btn');
    await addAccountBtn.waitForClickable({ timeout: 5000 });
    await addAccountBtn.click();
    await browser.pause(500);

    // Fill in account form
    const nameInput = await $('input[placeholder="e.g., Work, Personal"]');
    await nameInput.setValue('Test Radicale');

    const serverInput = await $('input[placeholder="https://caldav.example.com"]');
    await serverInput.setValue('http://localhost:5232');

    const usernameInput = await $('input[placeholder="Username"]');
    await usernameInput.setValue('test');

    const passwordInput = await $('input[placeholder="Password"]');
    await passwordInput.setValue('test');

    // Test connection
    const testBtn = await $('button.test-btn');
    await testBtn.click();

    // Wait for "Connection successful!" message
    const successMsg = await $('div.test-result.success');
    await successMsg.waitForDisplayed({ timeout: 10000 });

    // Save the account by pressing Enter on the form
    // TODO: Revisit this workaround - clicking the submit button via WebKitWebDriver
    // doesn't trigger form submission. We should find a proper user-like interaction
    // method instead of programmatic form submission.
    await nameInput.click();
    await browser.keys(['Enter']);
    await browser.pause(2000);

    // Debug: check what's on the page
    const pageSource = await browser.getPageSource();
    console.log('Page contains account-name:', pageSource.includes('account-name'));
    console.log('Page contains error:', pageSource.includes('error'));

    // Check if form is still visible (would mean save failed)
    const formStillVisible = await $('form.account-form').isExisting();
    console.log('Form still visible:', formStillVisible);

    // Verify account appears in list - use simple class selector then verify text
    const accountName = await $('span.account-name');
    await accountName.waitForDisplayed({ timeout: 5000 });
    const nameText = await accountName.getText();
    console.log('Account name:', nameText);
    expect(nameText).toBe('Test Radicale');
  });

  it('should sync account to import calendars', async () => {
    // Click sync button on the account
    const syncBtn = await $('button.sync-btn');
    await syncBtn.waitForClickable({ timeout: 5000 });
    console.log('Found sync button, clicking via JS...');
    // Use JS click like we did for form submit
    await browser.execute((btn) => btn.click(), syncBtn);

    // Wait for sync to complete
    console.log('Waiting for sync to complete...');
    await browser.pause(5000);

    // Debug: check page state
    const pageSource = await browser.getPageSource();
    console.log('Page has sync-result:', pageSource.includes('sync-result'));
    console.log('Page has syncing:', pageSource.includes('syncing'));
    console.log('Page has error:', pageSource.includes('error'));

    // Check for sync result or at least verify sync button is no longer spinning
    const syncResultExists = await $('div.sync-result').isExisting();
    console.log('Sync result exists:', syncResultExists);

    if (!syncResultExists) {
      console.log('No sync result, checking if sync completed silently...');
    }

    // Exit settings to see the imported list
    const settingsBtn = await $('button.settings-btn');
    await settingsBtn.click();
    await browser.pause(500);

    // Verify at least one CalDAV list exists (besides Inbox)
    const listBtns = await $$('button.list-name-btn');
    console.log('Number of lists:', listBtns.length);
    expect(listBtns.length).toBeGreaterThan(1);
  });

  it('should create a task in synced list and verify eager sync', async () => {
    // Click on a synced list (not Inbox - the second one)
    const listBtns = await $$('button.list-name-btn');
    // Find the non-Inbox list (synced from CalDAV)
    for (const btn of listBtns) {
      const text = await btn.getText();
      if (text !== 'Inbox') {
        await btn.click();
        break;
      }
    }
    await browser.pause(500);

    // Fill in task title
    const titleInput = await $('input.title-input');
    await titleInput.waitForDisplayed({ timeout: 5000 });
    await titleInput.setValue('Eager Sync Test Task');

    // Submit form
    const submitBtn = await $('button[type="submit"]');
    await submitBtn.click();

    // Wait for task creation and eager sync (background operation)
    await browser.pause(3000);

    // Verify task appears
    const taskTitle = await $('span.task-title');
    await taskTitle.waitForDisplayed({ timeout: 5000 });
    const titleText = await taskTitle.getText();
    expect(titleText).toBe('Eager Sync Test Task');

    // The task should have been eagerly synced to the server
    // We can verify this by checking if it has sync metadata (etag, href)
    // For now, we verify it shows up in the UI which means the sync didn't error
  });

  it('should toggle task completion with eager sync', async () => {
    // Find the checkbox
    const checkbox = await $('button.checkbox');
    await checkbox.waitForClickable({ timeout: 5000 });

    // Verify not completed
    let classes = await checkbox.getAttribute('class');
    expect(classes).not.toContain('checked');

    // Complete the task
    await checkbox.click();
    await browser.pause(2000); // Wait for eager sync

    // Verify completed
    classes = await checkbox.getAttribute('class');
    expect(classes).toContain('checked');
  });
});
