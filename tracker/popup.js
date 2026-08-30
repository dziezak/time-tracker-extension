const topListEl = document.getElementById('topList');
const statusDiv = document.getElementById('status');

function formatTime(totalSenconds) {
  const hours = Math.floor(totalSenconds / 3600);
  const minutes = Math.floor((totalSenconds % 3600) / 60);
  const seconds = totalSenconds % 60;
  if (hours > 0 ) return `${hours}h ${minutes}m ${seconds}s`;
  if (minutes > 0) return `${minutes}m ${seconds}s`;
  return `${seconds}s`;
}

async function renderTop5(){
  const data = await browser.storage.local.get("timeData");
  const timeData = data.timeData || {};

  const sortedDomains = Object.entries(timeData)
  .sort((a, b) => b[1] - a[1])
  .slice(0, 5);

  topListEl.innerHTML = '';

  if (sortedDomains.length === 0) {
    topListEl.innerHTML = `<li class="empty-msg">Brak danych o aktywnosci</li>`;
    return;
  }

  sortedDomains.forEach(([domain, seconds]) => {
    const li = document.createElement('li');
    li.className = 'top-item';
    li.innerHTML = `
      <span class="domain" title="${domain}">${domain}</span>
      <span class="time">${formatTime(seconds)}</span>
    `;

    topListEl.appendChild(li);
  });
}

document.getElementById('exportBtn').addEventListener('click', async () => {
  try {
    const data = await browser.storage.local.get("timeData");
    const timeData = data.timeData;

    if (!timeData || Object.keys(timeData).length === 0) {
      statusDiv.style.color = "red";
      statusDiv.textContent = "Brak danych!";
      return;
    }

    const jsonString = JSON.stringify(timeData, null, 2);
    const blob = new Blob([jsonString], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    
    const a = document.createElement("a");
    a.href = url;
    a.download = `time-data-${new Date().toISOString().slice(0,10)}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    statusDiv.style.color = "green";
    statusDiv.textContent = "Pobrano plik!";
  } catch (err) {
    statusDiv.style.color = "red";
    statusDiv.textContent = "Blad: " + err.message;
  }
});

document.getElementById('resetBtn').addEventListener('click', async () => {
  await browser.storage.local.set({ timeData: {} });
  statusDiv.style.color = "orange";
  statusDiv.textContent = "Wyczyszczono dane.";
});

renderTop5();