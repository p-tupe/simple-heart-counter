(() => {
  const shcEl = document.querySelector("#shc");
  if (!shcEl) {
    console.error("simple-heart-counter: no #shc element found");
    return;
  }

  const baseURL = shcEl.attributes.getNamedItem("data-url");
  if (!baseURL || !baseURL.value) {
    console.error("simple-heart-counter: no data-url attribute found");
    return;
  }
  const shcURL = baseURL.value + "/count";
  let delta = 0;

  const countEl = document.createElement("span");
  countEl.id = "shc-count";
  shcEl.appendChild(countEl);
  update_count(shcURL, delta, countEl, shcEl);

  shcEl.addEventListener("click", () => {
    if (shcEl.classList.contains("shc-clicked")) delta = -1;
    else delta = 1;
    update_count(shcURL, delta, countEl, shcEl);
  });
})();

function update_count(shcURL, delta, countEl, shcEl) {
  fetch(shcURL, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ url: window.location.toString(), delta }),
  })
    .then((d) => {
      if (d.status != 200) {
        throw Error("count request failed with status ", d.statusText);
      }
      return d.json();
    })
    .then(({ data: { count, clicked } }) => {
      countEl.textContent = count;
      if (clicked) {
        shcEl.classList.add("shc-clicked");
      } else {
        shcEl.classList.remove("shc-clicked");
      }
    })
    .catch((e) => console.error("simple-heart-counter:", e));
}
