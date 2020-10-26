const { h, render } = require("preact");

const t = require("../i18n").context("dialog");

let hiddenStyle = {
  opacity: 0,
  pointerEvents: "none",
};

exports.showMessageBox = function (
  message,
  type = "info",
  buttons = [t("OK")],
  cancelId = 0
) {
  if (buttons.length <= 1) {
    alert(message);
    return 0;
  } else {
    return confirm(message) ? 0 : cancelId;
  }
};

exports.showSaveDialog = function (options, callback) {
  let { type, name, content } = options;
  let href = `data:${type};charset=utf-8,${encodeURIComponent(content)}`;

  let element = render(
    h("a", {
      href,
      style: hiddenStyle,
      download: name,
    }),
    document.body
  );

  element.click();
  element.remove();
};
