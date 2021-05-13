/**
 * Enum representing game visibility
 */
const Visibility = {
  PUBLIC: "Public",
  PRIVATE: "Private",
};

/**
 * Enum representing the status of an initial connection
 * attempt from Sabaki client to Bugout gateway
 */
const ConnectionState = {
  DISCONNECTED: 0,
  IN_PROGRESS: 1,
  CONNECTED: 2,
  FAILED: 3,
};

const ColorPref = {
  BLACK: "Black",
  WHITE: "White",
  ANY: "Any",
};

const EntryMethod = {
  FIND_PUBLIC: 1,
  CREATE_PRIVATE: 2,
  JOIN_PRIVATE: 3,
  PLAY_BOT: 4,
};

const IdleStatus = {
  IDLE: "Idle",
  BOOTING: "Booting",
  ONLINE: "Online",
};

const BLACK = "B";
const WHITE = "W";

const Color = {
  BLACK,
  WHITE,
};

const BoardSize = {
  NINE: 9,
  THIRTEEN: 13,
  NINETEEN: 19,
};

/** Gateway uses this rep */
const Player = {
  BLACK: "BLACK",
  WHITE: "WHITE",
};

const Bot = {
  KATAGO_ONE_STAR: "KataGoOneStar",
  KATAGO_TWO_STARS: "KataGoTwoStars",
  KATAGO_THREE_STARS: "KataGoThreeStars",
  KATAGO_FOUR_STARS: "KataGoFourStars",
};

/** private to isValidGameId */
const MIN_ID_LENGTH = 4;
/** private to isValidGameId */
const MAX_ID_LENGTH = 30;
/** private to isValidGameId */
const re = new RegExp(/^[a-zA-Z0-9]+$/, "m");

const isValidGameId = (p) =>
  p && p.length >= MIN_ID_LENGTH && p.length <= MAX_ID_LENGTH && re.test(p);

const joinPrivateGameParam = () => {
  let urlParams = new URLSearchParams(window.location.search);

  if (urlParams.has("join") && isValidGameId(urlParams.get("join"))) {
    return { join: true, gameId: urlParams.get("join") };
  } else {
    return { join: false };
  }
};

const registerBoardSizeEvents = (app) => {
  const setSize = (boardSize) => {
    if (boardSize) {
      let { gameTrees, gameIndex } = app.state;
      let tree = gameTrees[gameIndex];
      app.setGameInfo(tree, { size: [boardSize, boardSize] });
    }
  };

  app.events.on("choose-board-size", ({ boardSize }) => setSize(boardSize));

  app.events.on("bugout-game-ready", ({ boardSize }) => setSize(boardSize));
};

const registerUndoEvent = (app) => {
  app.events.on("bugout-move-undone", () => {
    app.onMoveUndone();
  });
};

const registerReconnectEvents = (app) => {
  app.events.on("websocket-closed", () =>
    app.setState({
      multiplayer: {
        ...app.state.multiplayer,
        reconnectionState: ConnectionState.DISCONNECTED,
        reconnectDialog: true,
      },
    })
  );

  app.events.on("websocket-connecting", () =>
    app.setState({
      multiplayer: {
        ...app.state.multiplayer,
        reconnectionState: ConnectionState.IN_PROGRESS,
        reconnectDialog: true, // we've already connected once
      },
    })
  );

  app.events.on("websocket-error", () =>
    app.setState({
      multiplayer: {
        ...app.state.multiplayer,
        reconnectionState: ConnectionState.FAILED,
        reconnectDialog: true,
      },
    })
  );

  // The name differs since we're interested in a logical
  // reconnect, not simply a connection to the websocket.
  // We know that we have a valid game ID in hand.
  app.events.on("websocket-open", () => {
    app.setState({
      multiplayer: {
        ...app.state.multiplayer,
        reconnectionState: ConnectionState.CONNECTED,
      },
    });

    let dialogDurationMs = 1000;

    setTimeout(
      () =>
        app.setState({
          multiplayer: {
            ...app.state.multiplayer,
            reconnectDialog: false,
            playerUp: undefined,
          },
        }),
      dialogDurationMs
    );
  });
};

const emitReadyState = (ws, events) => {
  switch (ws.readyState) {
    case 0:
      events.emit("websocket-connecting");
      break;
    case 1:
      events.emit("websocket-open");
      break;
    case 2:
      events.emit("websocket-closed");
      break;
    case 3:
      events.emit("websocket-closed");
      break;
  }
};

const placeholderColor = Player.BLACK;

const load = () => {
  let engine = { path: "/bugout", args: "" };
  let jp = joinPrivateGameParam();
  let readyToEnter = (state) =>
    state.multiplayer &&
    (state.multiplayer.connectionState == undefined ||
      (state.multiplayer.connectionState < ConnectionState.IN_PROGRESS) &
        !state.multiplayer.reconnectDialog) &&
    (state.multiplayer.entryMethod || jp.join);
  return {
    joinPrivateGame: jp,
    engine,
    announceTurn: (gameTree, treePosition, events) =>
      events.emit("bugout-turn", {
        turn: [...gameTree.listNodesVertically(treePosition, -1, {})].length,
      }),
    attach: (appAttachEngines, playerColor) => {
      if (playerColor === WHITE) {
        appAttachEngines(engine, null);
      } else {
        appAttachEngines(null, engine);
      }
    },
    playerToColor: (player) => (player == Player.BLACK ? BLACK : WHITE),
    enterGame: (app, state) => {
      if (readyToEnter(state)) {
        app.setState({
          multiplayer: {
            ...app.state.multiplayer,
            connectionState: ConnectionState.IN_PROGRESS,
          },
        });

        app.detachEngines();

        app.bugout.attach((a, b) => {
          app.attachEngines(a, b);

          if (app.state.attachedEngines === [null, null]) {
            app.setState({
              multiplayer: {
                ...app.state.multiplayer,
                connectionState: ConnectionState.FAILED,
              },
            });
            throw Exception("multiplayer connect failed");
          } else {
            app.setState({
              multiplayer: {
                ...app.state.multiplayer,
                connectionState: ConnectionState.CONNECTED,
                reconnectDialog: false, // We just now connected for the first time
              },
            });

            app.events.once("your-color", ({ yourColor }) => {
              if (yourColor === Player.WHITE) {
                app.generateMove({ firstMove: true });
              }
            });

            app.events.on("human-color-selected", ({ humanColor }) => {
              if (humanColor[0].toUpperCase() === "W") {
                app.generateMove({ firstMove: true });
              }
            });

            registerReconnectEvents(app);
            registerBoardSizeEvents(app);
            registerUndoEvent(app);
          }
        }, placeholderColor);
      }
    },
  };
};

exports.emitReadyState = emitReadyState;
exports.load = load;
exports.Visibility = Visibility;
exports.ConnectionState = ConnectionState;
exports.ColorPref = ColorPref;
exports.Color = Color;
exports.EntryMethod = EntryMethod;
exports.Player = Player;
exports.IdleStatus = IdleStatus;
exports.BoardSize = BoardSize;
exports.Bot = Bot;
