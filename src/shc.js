(() => {
  const shc = document.querySelector("#shc");
  if (!shc) {
    console.error("simple-heart-counter: no #shc element found");
    return;
  }

  const baseURL = shc.attributes.getNamedItem("data-url");
  if (!baseURL || !baseURL.value) {
    console.error("simple-heart-counter: no data-url attribute found");
    return;
  }

  const countEl = document.createElement("span");
  countEl.id = "shc-count";
  shc.appendChild(countEl);

  fetch(baseURL.value + "/count", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ url: window.location.toString() }),
  })
    .then((d) => {
      if (d.status != 200) {
        throw Error("count request failed with status ", d.statusText);
      }
      return d.json();
    })
    .then(({ data: { count, clicked } }) => {
      countEl.textContent = count;
      if (clicked) shc.classList.add("shc-clicked");
    })
    .catch((e) => console.error("simple-heart-counter:", e));

  shc.addEventListener("click", () => {
    const currCount = Number(countEl.textContent) || 0;
    let url = baseURL.value + "/count";
    if (shc.classList.contains("shc-clicked")) {
      countEl.textContent = currCount - 1;
      shc.classList.remove("shc-clicked");
      url += "/decrement";
    } else {
      countEl.textContent = currCount + 1;
      shc.classList.add("shc-clicked");
      url += "/increment";
    }

    fetch(url, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ url: window.location.toString() }),
    }).catch((e) => console.error("simple-heart-counter:", e));
  });
})();
