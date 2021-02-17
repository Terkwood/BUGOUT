const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

class InvalidLinkModal extends Component {
  constructor() {
    super();
  }

  render({ id = "invalid-link-modal" }) {
    return h(
      Dialog,
      {
        id,
        isOpen: true,
      },
      h(Dialog.Header, null, "Invalid Link"),
      h(Dialog.Body, null, "This link is no longer available."),
      h(
        Dialog.Footer,
        null,
        h(
          Dialog.FooterButton,
          {
            accept: true,
            onClick: () => {
              location.reload();
            },
          },
          "START OVER"
        )
      )
    );
  }
}

export default InvalidLinkModal;
