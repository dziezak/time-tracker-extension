let activeDomain = null;

async function updateActiveTab() {
  try {
    const tabs = await browser.tabs.query({ active: true, currentWindow: true });
    if (tabs.length > 0 && tabs[0].url && tabs[0].url.startsWith("http")) {
      const url = new URL(tabs[0].url);
      activeDomain = url.hostname;
    } else {
      activeDomain = null;
    }
  } catch (err) {
    activeDomain = null;
  }
}

async function incrementTime() {
  if (!activeDomain) return;

  const data = await browser.storage.local.get("timeData");
  const timeData = data.timeData || {};

  timeData[activeDomain] = (timeData[activeDomain] || 0) + 1;

  await browser.storage.local.set({ timeData });
}

browser.tabs.onActivated.addListener(updateActiveTab);
browser.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (tab.active) updateActiveTab();
});
browser.windows.onFocusChanged.addListener(updateActiveTab);

setInterval(incrementTime, 1000);