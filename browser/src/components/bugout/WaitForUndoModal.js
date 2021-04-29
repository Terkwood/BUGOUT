const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

class WaitForUndoModal extends Component {
  constructor() {
    super();
    this.state = {
      showWait: false,
      showReject: false
    };

    // From GTP.js
    sabaki.events.on(
      "bugout-wait-for-undo",
      ({ showWait, showReject }) => {
        this.setState({ showWait, showReject });
      }
    );
  }

  renderWait(id) {
    return h(
        Dialog,
        {
          id,
          isOpen: true,
        },
        h(Dialog.Header, null, "Undo Move"),
        h(Dialog.Body, null, "Please wait for the server."),
        h(Dialog.Footer, null)
      );
  }

  renderReject(id) {
    return h(
        Dialog,
        {
          id,
          isOpen: true,
        },
        h(Dialog.Header, null, "Undo Move"),
        h(Dialog.Body, null, "Undo failed."),
        h(
            Dialog.Footer,
            null,
            h(
              Dialog.FooterButton,
              {
                accept: true,
                onClick: () => {
                  this.setState({ showReject: false, showWait: false });
                },
              },
              "OK"
            )
          ),
      );
  }

  render({ id = "wait-for-undo-modal" }) {
    let empty = h("div", { id });
    let { showWait, showReject } = this.state;
 
    if (showWait) {
        return this.renderWait(id);
    } else if (showReject) {
        return this.renderReject(id);
    } else {
        return empty;
    }
  }
}

export default WaitForUndoModal;
