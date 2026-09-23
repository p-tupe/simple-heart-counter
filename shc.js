(() => {
  const shc = document.querySelector("#shc");
  if (!shc) {
    console.error("no #shc element found");
    return;
  }

  const baseURL = shc.attributes.getNamedItem("data-url");
  if (!baseURL || !baseURL.value) {
    console.error("no data-url attribute found");
    return;
  }

  let countEl = document.querySelector("#shc-count");
  if (!countEl) {
    countEl = document.createElement("span");
    countEl.id = "shc-count";
    shc.appendChild(countEl);
  }

  fetch(baseURL.value + "/count", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ url: window.location.toString() }),
  })
    .then((d) => {
      if (d.status != 200) {
        throw Error("Request failed: ", d.statusText);
      }
      const data = d.json();
      return data;
    })
    .then(({ data: { count, clicked } }) => {
      countEl.textContent = count;
      if (clicked) {
        shc.classList.add("shc-clicked");
        shc.disabled = true;
      }
    })
    .catch(console.error);

  shc.addEventListener("click", () => {
    // TODO: add decrement instead of disabling
    const currCount = Number(countEl.textContent) || 0;
    countEl.textContent = currCount + 1;
    shc.classList.add("shc-clicked");
    shc.disabled = true;

    fetch(baseURL.value + "/count/increment", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ url: window.location.toString() }),
    }).catch(console.error);
  });
})();
