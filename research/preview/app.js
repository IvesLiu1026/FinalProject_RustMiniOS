const navButtons = Array.from(document.querySelectorAll(".nav-btn"));
const screens = Array.from(document.querySelectorAll("[data-ui-screen]"));

function setScreen(name) {
  navButtons.forEach((button) => {
    button.classList.toggle("is-active", button.dataset.screen === name);
  });

  screens.forEach((screen) => {
    screen.classList.toggle("hidden", screen.dataset.uiScreen !== name);
  });
}

navButtons.forEach((button) => {
  button.addEventListener("click", () => setScreen(button.dataset.screen));
});

setScreen("home");
