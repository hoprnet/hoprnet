let views = null;

const changeTab = name => {
  // if views object doesn't exist create it
  if (!views) {
    views = {
      stats: {
        container: document.getElementById("stats-container"),
        tab: document.getElementById("stats-tab")
      },
      stake: {
        container: document.getElementById("stake-container"),
        tab: document.getElementById("stake-tab")
      },
      votes: {
        container: document.getElementById("votes-container"),
        tab: document.getElementById("votes-tab")
      }
    };
  }

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

const onTabClick = changeTab;

function toggleTheme() {
  document.getElementById("html").classList.toggle("theme-light");
  document.getElementById("html").classList.toggle("theme-dark");
  toggleIcon();
}

function toggleIcon() {
  document.getElementById("sun").classList.toggle("do-not-display");
  document.getElementById("moon").classList.toggle("do-not-display");
}
