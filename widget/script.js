// store elements and avoid multiple calls to dom
const elements = new Map();

// get element and update cache
const getElementById = id => {
  if (elements.has(id)) return elements.get(id);

  const element = document.getElementById(id);
  elements.set(id, element);

  return element;
};

const changeTab = name => {
  const views = {
    stats: {
      container: getElementById("stats-container"),
      tab: getElementById("stats-tab")
    },
    stake: {
      container: getElementById("stake-container"),
      tab: getElementById("stake-tab")
    },
    votes: {
      container: getElementById("votes-container"),
      tab: getElementById("votes-tab")
    }
  };

  // loop through views and update the styles
  for (const tabName in views) {
    const { container, tab } = views[tabName];

    if (tabName === name) {
      container.classList.remove("hidden");
      tab.classList.add("tab-active");
    } else {
      container.classList.add("hidden");
      tab.classList.remove("tab-active");
    }
  }
};

function toggleTheme() {
  // swap colors
  getElementById("html").classList.toggle("theme-light");
  getElementById("html").classList.toggle("theme-dark");

  // swap visibility
  getElementById("sun").classList.toggle("hidden");
  getElementById("moon").classList.toggle("hidden");
}

const onTabClick = changeTab;
const onToggleTheme = toggleTheme;
