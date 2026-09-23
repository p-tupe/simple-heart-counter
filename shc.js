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
      return d.json();
    })
    .then(({ count }) => {
      countEl.textContent = count;
    })
    .catch((e) => {
      console.error(e);
    });

  shc.addEventListener("click", () => {
    console.log("Incrementing count...")
    const currCount = Number(countEl.textContent) || 0;
    countEl.textContent = currCount + 1;

    fetch(baseURL.value + "/count/increment", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ url: window.location.toString() }),
    }).catch((e) => {
      console.error(e);
    });
  });
})();
