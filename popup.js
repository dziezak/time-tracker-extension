const statusDiv = document.getElementById('status');

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