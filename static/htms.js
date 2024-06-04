document.querySelectorAll("form[x-get], form[x-post]").forEach((form) => {
  const replaceTargets = form.getAttribute("x-replace")?.split(" ");
  const pushUrl = form.hasAttribute("x-push-url");
  const updateTargets = [];
  form.addEventListener("change", (input) => {
    let url = form.getAttribute("x-get") || form.getAttribute("x-post");
    let method = "get";
    if (form.hasAttribute("x-post")) {
      method = "post";
    }
    const searchParams = new URLSearchParams(new FormData(form)).toString();
    url = url + `?${searchParams.toString()}`
    if (pushUrl) { history.pushState({}, "", url); }

    request(method, url)
      .then((text) => {
        updateDom(text, replaceTargets, updateTargets);
      })
      .catch((err) => {
        console.error(err);
      });
  });
});

function hitTargets(dom, targets, merge) {
  targets.forEach((target) => {
    const selector = `#${target}`;
    const next = dom.querySelector(selector);
    const current = document.querySelector(selector);

    if(!current) { return; }

    switch(merge) {
      case "replace":
        current.replaceWith(next);
        break;
      case "update":
        current.innerHTML = "";
        current.appendChild(next);
        break;
    }
  });
}

function updateDom(html, replaceTargets, updateTargets) {
    const dom = new DOMParser().parseFromString(html, "text/html");

    if(replaceTargets) {
      hitTargets(dom, replaceTargets, "replace");
    }

    if(updateTargets) {
      hitTargets(dom, updateTargets, "update");
    }
}

async function request(method, url, body) {
  let options = {
    method: method,
    redirect: "follow",
    headers: {
      "Content-Type": "application/json",
      "X-Request": "true"
    }
  };
  if(body) {
    options.body = body;
  }
  const response = await fetch(url, options);
  if(response.ok) {
    if(response.redirected) {
      history.pushState({}, "", response.url);
    }
    return await response.text();
  } else {
    return;
  }
}

