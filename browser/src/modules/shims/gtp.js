/// BUGOUT support for "gtp-like" multiplayer coordination

const EventEmitter = require("events");
const Board = require("../board");
const RobustWebSocket = require("robust-websocket");
const uuidv4 = require("uuid/v4");

const ClientId = require("../multiplayer/clientid");
const {
  IdleStatus,
  EntryMethod,
  emitReadyState,
  Player,
} = require("../multiplayer/bugout");

// for dev: host port 33012 should be mapped to container 3012
const GATEWAY_HOST_LOCAL = "ws://localhost:33012/gateway";
const GATEWAY_HOST_REMOTE = "wss://your.host.here:443/gateway";
const GATEWAY_HOST = GATEWAY_HOST_LOCAL;

const GATEWAY_BEEP_TIMEOUT_MS = 13333;

const IDLE_STATUS_POLL_MS = 1000;

const DEFAULT_BOARD_SIZE = 19;

class Controller extends EventEmitter {
  constructor(
    path,
    args = [],
    spawnOptions = {
      joinPrivateGame: { join: false },
      entryMethod: EntryMethod.FIND_PUBLIC,
    }
  ) {
    super();

    this.path = path;
    this.args = args;
    this.spawnOptions = spawnOptions;

    this._webSocketController = null;
  }

  get busy() {
    return this._webSocketController != null && this._webSocketController.busy;
  }

  start() {
    if (this._webSocketController != null) return;

    this._webSocketController = new WebSocketController(
      GATEWAY_HOST,
      this.spawnOptions
    );
    this._webSocketController.on("command-sent", (evt) =>
      this.emit("command-sent", evt)
    );
    this._webSocketController.on("response-received", (evt) =>
      this.emit("response-received", evt)
    );

    this.emit("started");
  }

  async stop(timeout = 3000) {
    if (this._webSocketController == null) return;

    return new Promise(async (resolve) => {
      this.kill();
      resolve();
    }, timeout);
  }

  kill() {
    if (this._webSocketController == null) return;

    this._webSocketController.stop();
  }

  async sendCommand(command, subscriber = () => {}) {
    if (this.webSocket == null) this.start();

    return await this._webSocketController.sendCommand(command, subscriber);
  }
}

const Command = {
  fromString: function (input) {
    input = input.replace(/#.*?$/, "").trim();

    let inputs = input.split(/\s+/);
    let id = parseInt(inputs[0], 10);

    if (!isNaN(id) && id + "" === inputs[0]) inputs.shift();
    else id = null;

    let [name, ...args] = inputs;
    return { id, name, args };
  },
  toString: function ({ id = null, name, args = [] }) {
    return `${id != null ? id : ""} ${name} ${args.join(" ")}`.trim();
  },
};

const letterToPlayer = (letter) => (letter === "B" ? "BLACK" : "WHITE");
const otherPlayer = (p) => (p[0] === "B" ? "WHITE" : "BLACK");

const FATAL_ERROR = "Fatal error";
const throwFatal = () => {
  alert(FATAL_ERROR);
  throw FATAL_ERROR;
};

/**
 * We've found that longer timeout allows for more stable
 * overall behavior.  It can take a long time for spotty
 * wi-fi to reestablish, and we don't want RobustWebSocket
 * giving up too early.
 */
const ROBUST_WEBSOCKET_TIMEOUT_MS = 300000;

const WEBSOCKET_HEALTH_DELAY_MS = 10000;
const WEBSOCKET_HEALTH_INTERVAL_MS = 100;

const opponentMoved = (msg, opponent) =>
  msg.type === "MoveMade" && msg.player === opponent;

const opponentQuit = (msg) => msg.type === "OpponentQuit";

class WebSocketController extends EventEmitter {
  constructor(webSocketAddress, spawnOptions) {
    super();

    this.board = new Board(DEFAULT_BOARD_SIZE, DEFAULT_BOARD_SIZE);

    sabaki.events.on("bot-selected", ({ bot }) => {
      this.bot = bot;
    });

    sabaki.events.on("human-color-selected", ({ humanColor }) => {
      if (this.deferredPlayBot) {
        this.deferredPlayBot(humanColor);
      }
    });

    sabaki.events.on("choose-board-size", ({ boardSize }) => {
      this.boardSize = boardSize;
      this.board = new Board(boardSize, boardSize);
      if (this.deferredCreatePrivate) {
        this.deferredCreatePrivate();
        this.deferredCreatePrivate = undefined;
      }
    });

    sabaki.events.on("bugout-game-ready", ({ boardSize }) => {
      this.boardSize = boardSize;
      this.board = new Board(boardSize, boardSize);
    });

    this.gameId = null;
    sabaki.events.on("resign", () => {
      this.gameId = null;
    });

    sabaki.events.on("undo", () => {
      console.log("GTP undo: TODO");
    })

    this.clientId = ClientId.fromStorage();

    this.beeping = true;
    setTimeout(() => this.beep(), GATEWAY_BEEP_TIMEOUT_MS);

    this.webSocketAddress = webSocketAddress;
    this.webSocket = new RobustWebSocket(webSocketAddress, null, {
      timeout: ROBUST_WEBSOCKET_TIMEOUT_MS,
    });

    let {
      joinPrivateGame,
      entryMethod,
      handleWaitForOpponent,
      handleYourColor,
    } = spawnOptions.multiplayer;
    this.joinPrivateGame = joinPrivateGame;
    this.entryMethod = entryMethod;

    console.log(
      "WS Controller Entry Method: " + JSON.stringify(this.entryMethod)
    );

    setTimeout(
      () =>
        setInterval(() => {
          emitReadyState(this.webSocket, sabaki.events);
        }, WEBSOCKET_HEALTH_INTERVAL_MS),
      WEBSOCKET_HEALTH_DELAY_MS
    );

    // We pass handleWaitForOpponent down so that it can 'stick'
    // to the incoming websocket message, even after an initial WFP
    // result is returned via findPublicGame() and createPrivateGame() funcs
    this.gatewayConn = new GatewayConn(
      this.webSocket,
      handleWaitForOpponent,
      handleYourColor
    );
    this.bugoutSync = new BugoutSync(this.webSocket);

    sabaki.events.on("sync-no-op", () => {
      // in case App.js is waiting for our play to resolve
      if (this.resolveMakeMove) {
        this.resolveMakeMove({ id: null, error: false });
        this.resolveMakeMove = undefined;
      }
    });
    sabaki.events.on(
      "sync-server-ahead",
      ({ type, replyTo, playerUp, turn, moves }) => {
        sabaki.generateMove();

        let syncLastMove = moves[moves.length - 1];
        let sabakiCoord = syncLastMove.coord
          ? this.board.vertex2coord([
              syncLastMove.coord.x,
              syncLastMove.coord.y,
            ])
          : "pass";

        if (this.resolveMoveMade) {
          console.log("Resolving outstanding move...");
          this.resolveMoveMade({
            id: null,
            content: sabakiCoord,
            error: false,
          });
        }

        let newPlayerUp = otherPlayer(playerUp);

        // In case white needs to dismiss its initial screen
        sabaki.events.emit("they-moved", { playerUp: newPlayerUp });

        // - In case we need to show that the opponent passed
        // - Used by BugoutSync to delay sync requests after move
        sabaki.events.emit("bugout-move-made", { coord: syncLastMove.coord });

        this.genMoveInProgress = false;
        sabaki.events.emit("gen-move-completed", { done: true });
      }
    );

    sabaki.events.on("bugout-turn", ({ turn }) => (this.turn = turn));

    this.webSocket.addEventListener("close", () => {
      this.removeMessageListener();
      console.log("WebSocket closed.");
      emitReadyState(this.webSocket, sabaki.events);
    });

    this.webSocket.addEventListener("error", (event) => {
      console.log(`WebSocket error ${JSON.stringify(event)}`);
      emitReadyState(this.webSocket, sabaki.events);
    });

    // support reconnect event
    this.webSocket.addEventListener("connecting", () => {
      this.removeMessageListener();
      emitReadyState(this.webSocket, sabaki.events);
    });

    this.webSocket.addEventListener("open", () => {
      this.removeMessageListener();
      emitReadyState(this.webSocket, sabaki.events);

      this.identifySelf().then((_idOk) => {
        // Bit of a dirty cheat: we know that playing
        // against the AI doesn't require the kafka
        // backend, so there's no need to wait for
        // that part of the system to start up.
        if (!this.gameId && this.entryMethod === EntryMethod.PLAY_BOT) {
          this.setupBotGame();
        } else {
          // Until https://github.com/Terkwood/BUGOUT/issues/174
          // is completed, we need to wait for the system to
          // come online when we're playing against a human being.
          this.waitForBugoutOnline().then((a, b) => this.onBugoutOnline(a, b));
        }
      });
    });
  }

  setupBotGame() {
    this.deferredPlayBot = (humanColor) =>
      this.gatewayConn
        .attachBot(this.boardSize, humanColor, this.bot)
        .then((reply, err) => {
          if (!err && reply.type === "BotAttached") {
            this.gameId = reply.gameId;

            let yourColor =
              humanColor.toUpperCase()[0] === "B" ? Player.BLACK : Player.WHITE;

            sabaki.events.emit("your-color", { yourColor });
          } else {
            throwFatal();
          }
        });
  }

  onBugoutOnline(_wrc, _werr) {
    if (!this.gameId && this.entryMethod === EntryMethod.FIND_PUBLIC) {
      this.gatewayConn.findPublicGame().then((reply, err) => {
        if (!err && reply.type === "GameReady") {
          this.gameId = reply.gameId;
          this.bugoutSync.activate(reply.gameId);
        } else if (!err && reply.type == "WaitForOpponent") {
          this.gameId = reply.gameId;
          this.bugoutSync.activate(reply.gameId);
        } else {
          throwFatal();
        }
      });
    } else if (
      !this.gameId &&
      this.entryMethod === EntryMethod.CREATE_PRIVATE
    ) {
      this.deferredCreatePrivate = () =>
        this.gatewayConn
          .createPrivateGame(this.boardSize || DEFAULT_BOARD_SIZE)
          .then((reply, err) => {
            if (!err && reply.type == "WaitForOpponent") {
              this.gameId = reply.gameId;
              this.bugoutSync.activate(reply.gameId);
            } else if (!err && reply.type === "GameReady") {
              this.gameId = reply.gameId;
              this.bugoutSync.activate(reply.gameId);
            } else {
              throwFatal();
            }
          });
    } else if (
      !this.gameId &&
      this.entryMethod === EntryMethod.JOIN_PRIVATE &&
      this.joinPrivateGame.join
    ) {
      this.gatewayConn
        .joinPrivateGame(this.joinPrivateGame.gameId)
        .then((reply, err) => {
          if (!err && reply.type === "GameReady") {
            this.gameId = reply.gameId;
            this.bugoutSync.activate(reply.gameId);
          } else if (!err && reply.type == "PrivateGameRejected") {
            sabaki.events.emit("private-game-rejected");
          } else {
            throwFatal();
          }
        });
    } else {
      this.gatewayConn
        .reconnect(this.gameId, this.resolveMoveMade, this.board)
        .then((rc, err) => {
          if (!err) {
            console.log(`Reconnected! data: ${JSON.stringify(rc)}`);

            if (this.genMoveInProgress) {
              let provideHistoryCommand = {
                type: "ProvideHistory",
                gameId: this.gameId,
                reqId: uuidv4(),
              };

              this.webSocket.send(JSON.stringify(provideHistoryCommand));

              let onMove = (r) => {
                if (r && r.resolveWith) {
                  // the opponent moved
                  this.genMoveInProgress = false;
                  this.resolveMoveMade(r.resolveWith);
                }
              };
              this.listenForHistoryOrMove(this.opponent, onMove);
            } else {
              this.listenForMove(this.opponent, this.resolveMoveMade);
            }
          } else {
            throwFatal();
          }
        });
    }
  }

  removeMessageListener() {
    this.messageListener &&
      this.webSocket.removeEventListener("message", this.messageListener);
  }

  updateMessageListener(listener) {
    if (listener) {
      this.removeMessageListener();
      this.messageListener = listener;
      this.webSocket.addEventListener("message", listener);
    }
  }

  listenForHistoryOrMove(opponent, onMove) {
    // We only want this listener online so we don't double-count turns
    this.updateMessageListener((event) => {
      try {
        let msg = JSON.parse(event.data);
        console.log(`msg on the line ${JSON.stringify(msg)}`);
        console.log(`turn ${this.turn}`);

        if (
          msg.type === "HistoryProvided" &&
          msg.moves.length > 0 &&
          msg.moves[msg.moves.length - 1].player === opponent &&
          msg.moves[msg.moves.length - 1].turn === this.turn
        ) {
          let lastMove = msg.moves[msg.moves.length - 1];
          if (lastMove) {
            // they didn't pass
            let sabakiCoord = this.board.vertex2coord([
              lastMove.coord.x,
              lastMove.coord.y,
            ]);

            onMove({
              player: lastMove.player,
              resolveWith: { id: null, content: sabakiCoord, error: false },
            });
          } else {
            // This may fail.  Revisit after https://github.com/Terkwood/BUGOUT/issues/56
            onMove({
              player: lastMove.player,
              resolveWith: { id: null, content: null, error: false },
            });
          }
        } else if (opponentMoved(msg, opponent)) {
          this.handleMoveMade(msg, opponent);
        } else {
          console.log("Unknown message");

          // discard any other messages until we receive confirmation
          // from BUGOUT that the history was provided
        }
      } catch (err) {
        console.log(
          `Error processing websocket message (H): ${JSON.stringify(err)}`
        );
        onMove(undefined);
      }
    });
  }

  listenForMove(opponent, resolve) {
    this.resolveMoveMade = resolve;

    // We only want this listener online so we don't double-count turns
    this.updateMessageListener((event) => {
      try {
        let msg = JSON.parse(event.data);

        if (opponentMoved(msg, opponent)) {
          this.handleMoveMade(msg, opponent, resolve);
          this.genMoveInProgress = false;
          sabaki.events.emit("gen-move-completed", { done: true });
        } else if (opponentQuit(msg)) {
          this.handleOpponentQuit(resolve);
          this.genMoveInProgress = false;
          sabaki.events.emit("gen-move-completed", { done: true });
        }

        // discard any other messages until we receive confirmation
        // from BUGOUT that the move was made
      } catch (err) {
        console.log(
          `Error processing websocket message (M): ${JSON.stringify(err)}`
        );
        resolve({ id: null, content: "", error: true });
      }
    });
  }

  handleMoveMade(msg, opponent, resolve) {
    // Note that the 'pass' value is used in
    // enginesyncer.js, which also has a special
    // case for the 'resign' value
    // See https://github.com/Terkwood/BUGOUT/issues/153

    let sabakiCoord = msg.coord
      ? this.board.vertex2coord([msg.coord.x, msg.coord.y])
      : "pass";

    resolve({ id: null, content: sabakiCoord, error: false });

    let playerUp = otherPlayer(opponent);

    // In case white needs to dismiss its initial screen
    sabaki.events.emit("they-moved", { playerUp });

    // - In case we need to show that the opponent passed
    // - Also used by BugoutSync to delay sync requests after move
    sabaki.events.emit("bugout-move-made", msg);
  }

  handleOpponentQuit(resolve) {
    this.gameId = null;
    sabaki.events.emit("bugout-opponent-quit");
    sabaki.makeResign();
    sabaki.setMode("scoring");
    resolve({ id: null, error: false });
  }

  async sendCommand(command, subscriber = () => {}) {
    let isPassing = (v) => v[0] == 14 && isNaN(v[1]);

    let promise = new Promise((resolve, reject) => {
      if (!this.gameId) {
        console.log(`no game id: ignoring command ${JSON.stringify(command)}`);
        reject({ id: null, error: true });
      }

      if (command.name == "play") {
        let player = letterToPlayer(command.args[0]);
        this.opponent = otherPlayer(player);

        let v = this.board.coord2vertex(command.args[1]);

        let coord = isPassing(v) ? null : { x: v[0], y: v[1] };

        let makeMove = {
          type: "MakeMove",
          gameId: this.gameId,
          reqId: uuidv4(),
          player: player,
          coord: coord,
        };

        this.resolveMakeMove = resolve;
        // We only want this listener online so we don't double-count turns
        this.updateMessageListener((event) => {
          try {
            let msg = JSON.parse(event.data);
            if (msg.type === "MoveMade" && msg.replyTo === makeMove.reqId) {
              resolve({ id: null, error: false });
            }

            // discard any other messages until we receive confirmation
            // from BUGOUT that the move was made
          } catch (err) {
            console.log(
              `Error processing websocket message: ${JSON.stringify(err)}`
            );
            resolve({ ok: false });
          }
        });

        let payload = JSON.stringify(makeMove);

        this.webSocket.send(payload);

        // Sync will be delayed as a result
        sabaki.events.emit("bugout-make-move");
      } else if (command.name === "genmove") {
        let opponent = letterToPlayer(command.args[0]);
        this.opponent = opponent;

        this.listenForMove(opponent, resolve);
        this.genMoveInProgress = true;
      } else {
        resolve({ id: null, err: false });
      }
    });

    this.emit("command-sent", {
      command,
      subscribe: (f) => {
        let g = subscriber;
        subscriber = (x) => (f(x), g(x));
      },
      getResponse: () => promise,
    });

    return promise;
  }

  async beep() {
    if (this.beeping) {
      const pingMsg = { type: "Beep" };
      this.webSocket.send(JSON.stringify(pingMsg));
      setTimeout(() => this.beep(), GATEWAY_BEEP_TIMEOUT_MS);
    }
  }

  stop() {
    this.webSocket.close();
    this.beeping = false;
  }

  async identifySelf() {
    let command = {
      type: "Identify",
      clientId: this.clientId,
    };

    this.webSocket.send(JSON.stringify(command));

    return new Promise((resolve, reject) => {
      this.updateMessageListener((event) => {
        try {
          let msg = JSON.parse(event.data);

          if (msg.type === "IdentityAcknowledged") {
            this.removeMessageListener();

            resolve(msg);
          }
          // discard any other messages until we receive confirmation
          // from BUGOUT that the move was made
        } catch (err) {
          console.log(
            `Error processing ID ACK response: ${JSON.stringify(err)}`
          );
          reject();
        }
      });
    });
  }

  async waitForBugoutOnline() {
    sabaki.events.on("idle-status", (idleStatus) =>
      sabaki.setState({
        multiplayer: {
          ...sabaki.state.multiplayer,
          idleStatus,
        },
      })
    );

    this.pollBugoutOnline();

    return new Promise((resolve, reject) => {
      this.updateMessageListener((event) => {
        try {
          let msg = JSON.parse(event.data);

          if (
            msg.type === "IdleStatusProvided" &&
            msg.status === IdleStatus.ONLINE
          ) {
            this.removeMessageListener();

            this.idleStatus = { status: msg.status };
            if (this.idleStatusPoll) {
              clearInterval(this.idleStatusPoll);
            }
            sabaki.events.emit("idle-status", this.idleStatus);

            resolve(msg);
          } else if (
            msg.type === "IdleStatusProvided" &&
            msg.status === IdleStatus.IDLE
          ) {
            this.idleStatus = { status: msg.status, since: msg.since };
            sabaki.events.emit("idle-status", this.idleStatus);
          } else if (
            msg.type === "IdleStatusProvided" &&
            msg.status === IdleStatus.BOOTING
          ) {
            this.idleStatus = { status: msg.status, since: msg.since };
            sabaki.events.emit("idle-status", this.idleStatus);
          } else {
            throw "wait-error-halp";
          }

          // discard any other messages until we receive confirmation
          // from BUGOUT that the move was made
        } catch (err) {
          console.log(
            `Error processing idle status response: ${JSON.stringify(err)}`
          );
          reject();
        }
      });
    });
  }

  pollBugoutOnline() {
    let command = {
      type: "ProvideIdleStatus",
    };

    this.webSocket.send(JSON.stringify(command));

    this.idleStatusPoll = setInterval(() => {
      if (
        this.idleStatus &&
        this.idleStatus.status &&
        this.idleStatus.status !== IdleStatus.ONLINE
      ) {
        let command = {
          type: "ProvideIdleStatus",
        };

        this.webSocket.send(JSON.stringify(command));
      }
    }, IDLE_STATUS_POLL_MS);
  }
}

class GatewayConn {
  constructor(webSocket, handleWaitForOpponent, handleYourColor) {
    this.webSocket = webSocket;

    if (handleWaitForOpponent == undefined || handleYourColor == undefined) {
      throw Exception("malformed gateway conn");
    }

    // We manage handleWaitForOpponent at this level
    // so that the incoming websocket message triggers
    // a state update in App.js, even after an initial Wait event
    // has been handled by the WebsocketController
    this.handleWaitForOpponent = handleWaitForOpponent;

    this.handleYourColor = handleYourColor;

    sabaki.events.on("choose-color-pref", ({ colorPref }) =>
      this.chooseColorPref(colorPref)
    );

    sabaki.events.on("resign", () => this.quitGame());
  }

  async reconnect(gameId, resolveMoveMade, board) {
    return new Promise((resolve, reject) => {
      try {
        let reconnectCommand = {
          type: "Reconnect",
          gameId: gameId,
          reqId: uuidv4(),
        };

        this.webSocket.onmessage = (event) => {
          try {
            let msg = JSON.parse(event.data);
            if (msg.type === "Reconnected") {
              resolve({ playerUp: msg.playerUp });
            }

            // listens for _any_ player to move ...
            if (resolveMoveMade && msg.type == "MoveMade") {
              let sabakiCoord = board.vertex2coord([msg.coord.x, msg.coord.y]);

              resolveMoveMade({ id: null, content: sabakiCoord, error: false });
            }

            // discard any other messages
          } catch (err) {
            console.log(
              `Error processing websocket message (R): ${JSON.stringify(err)}`
            );
            resolve({ error: true });
          }
        };

        this.webSocket.send(JSON.stringify(reconnectCommand));
      } catch (err) {
        reject(err);
      }
    });
  }

  async attachBot(boardSize, humanColor, bot) {
    return new Promise((resolve, reject) => {
      let player = otherPlayer(humanColor);

      let requestPayload = {
        type: "AttachBot",
        boardSize,
        player,
        bot,
      };

      this.webSocket.addEventListener("message", (event) => {
        try {
          let msg = JSON.parse(event.data);

          if (msg.type === "BotAttached") {
            let isBotPlaying = msg.player === "BLACK";

            sabaki.events.emit("bugout-wait-for-bot", {
              isModalRelevant: true,
              isBotAttached: true,
              isBotPlaying,
            });

            if (isBotPlaying) {
              sabaki.events.once("gen-move-completed", () => {
                // Turn off the modal forever
                sabaki.events.emit("bugout-wait-for-bot", {
                  isModalRelevant: false,
                });
              });
            }

            // App.js wants to know about this as well
            sabaki.events.emit("bugout-bot-attached", msg);

            resolve(msg);
          }
          // discard any other messages
        } catch (err) {
          console.log(
            `Error processing websocket message: ${JSON.stringify(err)}`
          );
          reject();
        }
      });

      sabaki.events.emit("bugout-wait-for-bot", {
        isModalRelevant: true,
        isBotAttached: false,
      });
      this.webSocket.send(JSON.stringify(requestPayload));
    });
  }

  async findPublicGame() {
    return new Promise((resolve, reject) => {
      let requestPayload = {
        type: "FindPublicGame",
      };

      this.webSocket.addEventListener("message", (event) => {
        try {
          let msg = JSON.parse(event.data);

          if (msg.type === "GameReady") {
            resolve(msg);
            this.handleWaitForOpponent({ gap: false, hasEvent: false });
          } else if (msg.type === "WaitForOpponent") {
            resolve(msg);
            this.handleWaitForOpponent({
              gap: false,
              hasEvent: true,
              event: msg,
            });
          }
          // discard any other messages
        } catch (err) {
          console.log(
            `Error processing websocket message: ${JSON.stringify(err)}`
          );
          reject();
        }
      });

      // We want to show the modal while we wait for a response from gateway
      this.handleWaitForOpponent({ gap: true, hasEvent: false });
      this.webSocket.send(JSON.stringify(requestPayload));
    });
  }

  async createPrivateGame(boardSize) {
    return new Promise((resolve, reject) => {
      let requestPayload = {
        type: "CreatePrivateGame",
        boardSize,
      };

      this.webSocket.addEventListener("message", (event) => {
        try {
          let msg = JSON.parse(event.data);

          if (msg.type === "WaitForOpponent") {
            resolve(msg);
            this.handleWaitForOpponent({
              gap: false,
              hasEvent: true,
              event: msg,
            });
          } else if (msg.type === "GameReady") {
            // later ...
            resolve(msg);
            // turn off dialog
            this.handleWaitForOpponent({ gap: false, hasEvent: false });
            sabaki.events.emit("bugout-game-ready", msg);
          }
          // discard any other messages
        } catch (err) {
          console.log(
            `Error processing websocket message: ${JSON.stringify(err)}`
          );
          reject();
        }
      });

      // We want to show the modal while we wait for a response from gateway
      this.handleWaitForOpponent({ gap: true, hasEvent: false });
      this.webSocket.send(JSON.stringify(requestPayload));
    });
  }

  async joinPrivateGame(gameId) {
    return new Promise((resolve, reject) => {
      let requestPayload = {
        type: "JoinPrivateGame",
        gameId,
      };

      this.webSocket.addEventListener("message", (event) => {
        try {
          let msg = JSON.parse(event.data);

          if (msg.type === "GameReady") {
            resolve(msg);
            this.handleWaitForOpponent({ gap: false, hasEvent: false });
            sabaki.events.emit("bugout-game-ready", msg);
          } else if (msg.type === "PrivateGameRejected") {
            resolve(msg);
          }
          // discard any other messages
        } catch (err) {
          console.log(
            `Error processing websocket message: ${JSON.stringify(err)}`
          );
          reject();
        }
      });

      // We want to show the modal while we wait for a response from gateway
      this.handleWaitForOpponent({ gap: true, hasEvent: false });
      this.webSocket.send(JSON.stringify(requestPayload));
    });
  }

  async chooseColorPref(colorPref) {
    return new Promise((resolve, reject) => {
      let requestPayload = {
        type: "ChooseColorPref",
        colorPref,
      };

      // Let this listener stack
      this.webSocket.addEventListener("message", (event) => {
        try {
          let msg = JSON.parse(event.data);

          if (msg.type === "YourColor") {
            resolve(msg);
            this.handleYourColor({ wait: false, event: msg });
          }
          // discard any other messages
        } catch (err) {
          console.log(
            `Error processing websocket message: ${JSON.stringify(err)}`
          );
          reject();
        }
      });

      // We want to show a modal while we wait for a response from gateway
      this.handleYourColor({ wait: true });
      this.webSocket.send(JSON.stringify(requestPayload));
    });
  }

  async quitGame() {
    this.webSocket.send('{"type": "QuitGame"}');
  }
}

const SYNC_TIMEOUT_MS = 5000;
const SYNC_DELAY_MS = 7500;

class BugoutSync {
  constructor(webSocket) {
    this.webSocket = webSocket;
    this.activated = false;
    this.gameId = undefined;
    this.delayUntil = undefined;
    this.reqId = undefined;

    sabaki.events.on("bugout-move-made", () => this.delay());
    sabaki.events.on("bugout-make-move", () => this.delay());
  }

  activate(gameId) {
    this.gameId = gameId;
    this.delay();
    this.interval = setInterval(() => this.reqSync(), SYNC_TIMEOUT_MS);
    this.activated = true;
    console.log("Sync Activated");
  }

  delay() {
    this.reqId = undefined;
    this.delayUntil = Date.now() + SYNC_DELAY_MS;
  }

  reqSync() {
    if (this.activated && Date.now() > (this.delayUntil || 0)) {
      this.reqId = uuidv4();

      const payload = this.makePayload(this.reqId);
      this.webSocket.send(JSON.stringify(payload));
      this.updateMessageListener((event) => {
        try {
          let msg = JSON.parse(event.data);

          if (msg.type === "SyncReply" && this.reqId === msg.replyTo) {
            this.processReply(msg);
          }
        } catch (e) {
          console.log("sync err " + JSON.stringify(e));
        }
      });
    }
  }

  processReply(syncReply) {
    let { playerUp, lastMove, turn } = deriveLocalState();

    if (syncReply.turn === turn && syncReply.playerUp === playerUp) {
      sabaki.events.emit("sync-no-op");
    } else if (
      syncReply.turn - 1 === turn &&
      otherPlayer(syncReply.playerUp) === playerUp
    ) {
      console.log("!  SERVER IS AHEAD");
      sabaki.events.emit("sync-server-ahead", syncReply);
    } else if (
      syncReply.turn + 1 === turn &&
      syncReply.playerUp === otherPlayer(playerUp)
    ) {
      console.log(
        "!  SERVER IS BEHIND (and will catch up on next sync request)"
      );
    } else {
      console.log("!  SYNC: CRITICAL FAILURE");
      console.log(`   - syncReply: ${JSON.stringify(syncReply)}`);
      console.log(
        `   - local    : \n\t\t\tplayerUp ${JSON.stringify(
          playerUp
        )}\n\t\t\tturn ${JSON.stringify(turn)}\n\t\t\tlastMove ${JSON.stringify(
          lastMove
        )}\n\t\t\ttree ${JSON.stringify(tree)}`
      );
    }
  }

  makePayload(reqId) {
    let { playerUp, lastMove, turn } = deriveLocalState();

    return {
      type: "ReqSync",
      playerUp,
      reqId,
      turn,
      lastMove,
    };
  }

  removeMessageListener() {
    this.messageListener &&
      this.webSocket.removeEventListener("message", this.messageListener);
  }

  updateMessageListener(listener) {
    if (listener) {
      this.removeMessageListener();
      this.messageListener = listener;
      this.webSocket.addEventListener("message", listener);
    }
  }
}

const deriveLocalState = () => {
  let { gameTrees, gameIndex } = sabaki.state;
  let { currentPlayer } = sabaki.inferredState;

  let playerUp = interpretPlayerNum(currentPlayer);
  let tree = gameTrees[gameIndex];
  let lastMove = findLastMove(tree);
  let turn = lastMove == undefined ? 1 : lastMove.turn + 1;

  return { playerUp, lastMove, turn };
};

const interpretPlayerNum = (n) => (n === 1 ? "BLACK" : "WHITE");

const findLastMove = (tree) => {
  var bottom = false;

  if (
    tree === undefined ||
    tree.root === undefined ||
    tree.root.children === undefined ||
    tree.root.children.length === 0
  ) {
    return null;
  }

  // skip the top level game node
  var subtree = tree.root.children[0];
  var turn = 0;
  var lastMove = null;
  while (!bottom) {
    turn = turn + 1;

    if (subtree && subtree.data) {
      let blackTreeCoords = subtree.data.B;
      let whiteTreeCoords = subtree.data.W;

      var proceed = false;
      if (blackTreeCoords) {
        let coord = convertTreeCoord(blackTreeCoords);
        let player = "BLACK";
        lastMove = { turn, player, coord };
        proceed = true;
      } else if (whiteTreeCoords) {
        let coord = convertTreeCoord(whiteTreeCoords);
        let player = "WHITE";
        lastMove = { turn, player, coord };
        proceed = true;
      }

      if (proceed) {
        if (subtree.children && subtree.children.length > 0) {
          subtree = subtree.children[0];
        } else {
          bottom = true;
        }
      } else {
        bottom = true;
      }
    } else {
      bottom = true;
    }
  }
  return lastMove;
};

const convertTreeCoord = (treeCoords) => {
  const offset = 97;
  if (
    treeCoords === undefined ||
    treeCoords[0] === undefined ||
    treeCoords[0].length !== 2
  ) {
    return null;
  } else {
    return {
      x: treeCoords[0].charCodeAt(0) - offset,
      y: treeCoords[0].charCodeAt(1) - offset,
    };
  }
};

exports.Controller = Controller;
exports.Command = Command;
exports.letterToPlayer = letterToPlayer;
