const { h, Component } = require("preact");

// 🦹🏻‍ Bundle Bloat Protector
import Dialog from "preact-material-components/Dialog";

const { EntryMethod, IdleStatus } = require("../../modules/multiplayer/bugout");

const formatSince = (since) => {
  let secs = Math.floor((Date.now() - Date.parse(since)) / 1000);
  return `${secs}s`;
};

const IGNORED_ENTRY_METHODS = [EntryMethod.PLAY_BOT];

class IdleStatusModal extends Component {
  constructor() {
    super();
  }

  render({ id = "idle-status-modal", data }) {
    let empty = h("div", { id });

    if (undefined == data) {
      return empty;
    }

    let { idleStatus, entryMethod } = data;

    if (IGNORED_ENTRY_METHODS.includes(entryMethod)) {
      return empty;
    }

    if (undefined == idleStatus || undefined == idleStatus.status) {
      return h(
        Dialog,
        {
          id,
          isOpen: true,
        },
        h(Dialog.Header, null, "Please Wait"),
        h(Dialog.Body, null, "Checking system availability...")
      );
    }

    if (idleStatus.status === IdleStatus.ONLINE) {
      return empty;
    }

    if (idleStatus.status === IdleStatus.BOOTING) {
      return h(
        Dialog,
        {
          id,
          isOpen: true,
        },
        h(Dialog.Header, null, "Please Wait"),
        h(
          Dialog.Body,
          null,
          `BUGOUT is initializing (${formatSince(idleStatus.since)}).`
        )
      );
    }

    // IDLE
    return h(
      Dialog,
      {
        id,
        isOpen: true,
      },
      h(Dialog.Header, null, "Offline"),
      h(
        Dialog.Body,
        null,
        `BUGOUT has been idle ${formatSince(idleStatus.since)}.`
      )
    );
  }
}

export default IdleStatusModal;
