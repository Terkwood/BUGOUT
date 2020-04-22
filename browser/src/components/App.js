const EDITION = 'Aal'

const EventEmitter = require('events')
const {ipcRenderer, remote} = require('electron')
const {app} = remote
const {h, render, Component} = require('preact')
const classNames = require('classnames')

const MainView = require('./MainView')
const DrawerManager = require('./DrawerManager')

// BUGOUT 🦹🏻‍ Bundle Bloat Protector
import BoardSizeModal from './bugout/BoardSizeModal'
import GameLobbyModal from './bugout/WelcomeModal'
import IdleStatusModal from './bugout/IdleStatusModal'
import MultiplayerColorPrefModal from './bugout/MultiplayerColorPrefModal'
import OpponentPassedModal from './bugout/OpponentPassedModal'
import OpponentQuitModal from './bugout/OpponentQuitModal'
import PlayBotColorSelectionModal from './bugout/PlayBotColorSelectionModal'
import ReconnectModal from './bugout/ReconnectModal'
import WaitForBotModal from './bugout/WaitForBotModal'
import WaitForOpponentModal from './bugout/WaitForOpponentModal'
import WaitForYourColorModal from './bugout/WaitForYourColorModal'
import YourColorChosenModal from './bugout/YourColorChosenModal'

const deadstones = require('@sabaki/deadstones')
const sgf = require('@sabaki/sgf')
const influence = require('@sabaki/influence')

deadstones.useFetch('./node_modules/@sabaki/deadstones/wasm/deadstones_bg.wasm')

const i18n = require('../i18n')
const Board = require('../modules/board')
const EngineSyncer = require('../modules/enginesyncer')
const dialog = require('../modules/dialog')
const fileformats = require('../modules/fileformats')
const gametree = require('../modules/gametree')
const helper = require('../modules/helper')
const setting = remote.require('./setting')
const sound = require('../modules/sound')
const bugout = require('../modules/multiplayer/bugout')

class App extends Component {
    constructor() {
        super()

        this.bugout = bugout.load()

        window.sabaki = this

        let emptyTree = gametree.new()

        this.state = {
            mode: 'play',
            openDrawer: null,
            busy: 0,
            fullScreen: false,
            showMenuBar: null,
            zoomFactor: null,

            representedFilename: null,
            gameIndex: 0,
            gameTrees: [emptyTree],
            gameCurrents: [{}],
            treePosition: emptyTree.root.id,

            // Bars

            selectedTool: 'stone_1',
            scoringMethod: null,
            findText: '',
            findVertex: null,
            deadStones: [],
            blockedGuesses: [],

            // Goban

            highlightVertices: [],
            playVariation: null,
            showCoordinates: null,
            showMoveColorization: null,
            showMoveNumbers: setting.get('view.show_move_numbers'), // 😇BUGOUT😇
            showNextMoves: null,
            showSiblings: null,
            fuzzyStonePlacement: null,
            animateStonePlacement: null,

            // Sidebar
            showGameGraph: false, // 😇BUGOUT😇
            showCommentBox: false, // 😇BUGOUT😇
            sidebarWidth: 120, // 😇BUGOUT😇
            graphGridSize: null,
            graphNodeSize: null,

            // Engines

            engines: null,
            attachedEngines: [null, null],
            engineBusy: [false, false],
            engineCommands: [[], []],
            generatingMoves: false,
            analysisTreePosition: null,
            analysis: null,

            // Drawers
            preferencesTab: 'general'
        }

        this.events = new EventEmitter()
        this.appName = app.getName()
        this.version = app.getVersion()
        this.window = remote.getCurrentWindow()

        this.treeHash = this.generateTreeHash()
        this.attachedEngineSyncers = [null, null]

        this.historyPointer = 0
        this.history = []
        this.recordHistory()

        // Expose submodules

        this.modules = {Board, EngineSyncer, dialog, fileformats,
            gametree, helper, i18n, setting, sound}

        // Bind state to settings

        setting.events.on('change', ({key}) => this.updateSettingState(key))
        this.updateSettingState()

        // from GatewayConn
        this.events.on('bugout-bot-attached', ({ player }) =>
            this.setState({
                multiplayer: {
                    ...this.state.multiplayer,
                    botColor: player
                }
            })
        )

        console.log(`Welcome to Sabaki - BUGOUT ${EDITION} Edition`)
    }

    componentDidMount() {
        window.addEventListener('contextmenu', evt => {
            evt.preventDefault()
        })

        window.addEventListener('load', () => {
            this.events.emit('ready')
        })

        this.window.on('focus', () => {
            this.buildMenu()
        })

        this.window.on('resize', () => {
            clearTimeout(this.resizeId)

            this.resizeId = setTimeout(() => {
                if (!this.window.isMaximized() && !this.window.isMinimized() && !this.window.isFullScreen()) {
                    let [width, height] = this.window.getContentSize()
                    setting.set('window.width', width).set('window.height', height)
                }
            }, 1000)
        })

        // Handle keys

        document.addEventListener('keydown', evt => {
            if (evt.key === 'Escape') {
                if (this.state.openDrawer != null) {
                    this.closeDrawer()
                } else if (this.state.mode !== 'play') {
                    this.setMode('play')
                } else if (this.state.fullScreen) {
                    this.setState({fullScreen: false})
                }
            }
        })

        // Handle window closing

        window.addEventListener('beforeunload', evt => {
            evt.returnValue = ' '
        })

        this.newFile()
    }

    componentDidUpdate(_, prevState = {}) {
        // Update title

        let {basename} = require('path')
        let title = this.appName
        let {representedFilename, gameIndex, gameTrees} = this.state
        let t = i18n.context('app')

        if (representedFilename)
            title = basename(representedFilename)
        if (gameTrees.length > 1)
            title += ' — ' + t(p => `Game ${p.gameNumber}`, {
                gameNumber: gameIndex + 1
            })
        if (representedFilename)
            title += ' — ' + this.appName

        if (document.title !== title)
            document.title = title

        // Handle zoom factor

        if (prevState.zoomFactor !== this.state.zoomFactor) {
            this.window.webContents.setZoomFactor(this.state.zoomFactor)
        }
    }

    updateSettingState(key = null, {buildMenu = true} = {}) {
        let data = {
            'app.zoom_factor': 'zoomFactor',
            'view.show_coordinates': 'showCoordinates',
            'view.show_move_colorization': 'showMoveColorization',
            'view.show_move_numbers': 'showMoveNumbers',
            'view.show_next_moves': 'showNextMoves',
            'view.show_siblings': 'showSiblings',
            'view.fuzzy_stone_placement': 'fuzzyStonePlacement',
            'view.animated_stone_placement': 'animateStonePlacement',
            'graph.grid_size': 'graphGridSize',
            'graph.node_size': 'graphNodeSize',
            'engines.list': 'engines',
            'scoring.method': 'scoringMethod'
        }

        if (key == null) {
            for (let k in data) this.updateSettingState(k, {buildMenu: false})
            this.buildMenu()
            return
        }

        if (key in data) {
            if (buildMenu) this.buildMenu()
            this.setState({[data[key]]: setting.get(key)})
        }
    }

    waitForRender() {
        return new Promise(resolve => this.setState({}, resolve))
    }

    // User Interface

    buildMenu() {
        ipcRenderer.send('build-menu', {
            disableAll: this.state.busy > 0
        })
    }

    setMode(mode) {
        let stateChange = {mode}

        if (['scoring', 'estimator'].includes(mode)) {
            // Guess dead stones

            let {gameIndex, gameTrees, treePosition} = this.state
            let iterations = setting.get('score.estimator_iterations')
            let tree = gameTrees[gameIndex]

            deadstones.guess(gametree.getBoard(tree, treePosition).arrangement, {
                finished: mode === 'scoring',
                iterations
            }).then(result => {
                this.setState({deadStones: result})
            })
        }

        this.setState(stateChange)
        this.events.emit('modeChange')
    }

    openDrawer(drawer) {
        this.setState({openDrawer: drawer})
    }

    closeDrawer() {
        this.openDrawer(null)
    }

    setBusy(busy) {
        let diff = busy ? 1 : -1;
        this.setState(s => ({busy: Math.max(s.busy + diff, 0)}))
    }

    // History Management

    recordHistory({prevGameIndex, prevTreePosition} = {}) {
        let currentEntry = this.history[this.historyPointer]
        let newEntry = {
            gameIndex: this.state.gameIndex,
            gameTrees: this.state.gameTrees,
            treePosition: this.state.treePosition,
            timestamp: Date.now()
        }

        if (
            currentEntry != null
            && helper.shallowEquals(currentEntry.gameTrees, newEntry.gameTrees)
        ) return

        this.history = this.history.slice(-setting.get('edit.max_history_count'), this.historyPointer + 1)

        if (
            currentEntry != null
            && newEntry.timestamp - currentEntry.timestamp < setting.get('edit.history_batch_interval')
        ) {
            this.history[this.historyPointer] = newEntry
        } else {
            if (currentEntry != null && prevGameIndex != null && prevTreePosition != null) {
                currentEntry.gameIndex = prevGameIndex
                currentEntry.treePosition = prevTreePosition
            }

            this.history.push(newEntry)
            this.historyPointer = this.history.length - 1
        }
    }

    clearHistory() {
        this.history = []
        this.recordHistory()
    }

    checkoutHistory(historyPointer) {
        let entry = this.history[historyPointer]
        if (entry == null) return

        let gameTree = entry.gameTrees[entry.gameIndex]

        this.historyPointer = historyPointer
        this.setState({
            gameIndex: entry.gameIndex,
            gameTrees: entry.gameTrees,
            gameCurrents: entry.gameTrees.map(_ => ({}))
        })

        this.setCurrentTreePosition(gameTree, entry.treePosition, {clearCache: true})
    }

    // File Management

    getEmptyGameTree() {
        let handicap = setting.get('game.default_handicap')
        let size = setting.get('game.default_board_size').toString().split(':').map(x => +x)
        let [width, height] = [size[0], size.slice(-1)[0]]
        let handicapStones = new Board(width, height).getHandicapPlacement(handicap).map(sgf.stringifyVertex)

        let sizeInfo = width === height ? width.toString() : `${width}:${height}`
        let date = new Date()
        let dateInfo = sgf.stringifyDates([[date.getFullYear(), date.getMonth() + 1, date.getDate()]])

        return gametree.new().mutate(draft => {
            let rootData = {
                GM: ['1'], FF: ['4'], CA: ['UTF-8'],
                AP: [`${this.appName}:${this.version}`],
                KM: [setting.get('game.default_komi')],
                SZ: [sizeInfo], DT: [dateInfo]
            }

            if (handicapStones.length > 0) {
                Object.assign(rootData, {
                    HA: [handicap.toString()],
                    AB: handicapStones
                })
            }

            for (let prop in rootData) {
                draft.updateProperty(draft.root.id, prop, rootData[prop])
            }
        })
    }

    async newFile({playSound = false, showInfo = false} = {}) {
        let emptyTree = this.getEmptyGameTree()

        await this.loadGameTrees([emptyTree], {})

        if (showInfo) this.openDrawer('info')
        if (playSound) sound.playNewGame()
    }

    async loadContent(content, extension, options = {}) {
        this.setBusy(true)

        let t = i18n.context('app.file')
        let gameTrees = []
        let success = true
        let lastProgress = -1

        try {
            let fileFormatModule = fileformats.getModuleByExtension(extension)

            gameTrees = fileFormatModule.parse(content, evt => {
                if (evt.progress - lastProgress < 0.1) return
                this.window.setProgressBar(evt.progress)
                lastProgress = evt.progress
            })

            if (gameTrees.length == 0) throw true
        } catch (err) {
            dialog.showMessageBox(t('This file is unreadable.'), 'warning')
            success = false
        }

        if (success) {
            await this.loadGameTrees(gameTrees, options)
        }

        this.setBusy(false)
    }

    async loadGameTrees(gameTrees, {clearHistory = true} = {}) {
        this.setBusy(true)
        if (this.state.openDrawer !== 'gamechooser') this.closeDrawer()
        this.setMode('play')

        await helper.wait(setting.get('app.loadgame_delay'))

        if (gameTrees.length > 0) {
            this.detachEngines()

            this.setState({
                representedFilename: null,
                gameIndex: 0,
                gameTrees,
                gameCurrents: gameTrees.map(_ => ({}))
            })

            let [firstTree, ] = gameTrees
            this.setCurrentTreePosition(firstTree, firstTree.root.id, {clearCache: true})

            this.treeHash = this.generateTreeHash()
            this.fileHash = this.generateFileHash()

            if (clearHistory) this.clearHistory()
        }

        this.setBusy(false)
        this.window.setProgressBar(-1)
        this.events.emit('fileLoad')

        if (gameTrees.length > 1) {
            await helper.wait(setting.get('gamechooser.show_delay'))
            this.openDrawer('gamechooser')
        }
    }

    saveFile() {
        dialog.showSaveDialog({
            type: 'application/x-go-sgf',
            name: this.state.representedFilename || 'game.sgf',
            content: this.getSGF()
        })

        this.treeHash = this.generateTreeHash()
        this.fileHash = this.generateFileHash()

        return true
    }

    getSGF() {
        let {gameTrees} = this.state

        gameTrees = gameTrees.map(tree => tree.mutate(draft => {
            draft.updateProperty(draft.root.id, 'AP', [`${this.appName}:${this.version}`])
            draft.updateProperty(draft.root.id, 'CA', ['UTF-8'])
        }))

        this.setState({gameTrees})
        this.recordHistory()

        return sgf.stringify(gameTrees.map(tree => tree.root), {
            linebreak: setting.get('sgf.format_code') ? helper.linebreak : ''
        })
    }

    generateTreeHash() {
        return this.state.gameTrees.map(tree => gametree.getHash(tree)).join('-')
    }

    generateFileHash() {
    }

    askForSave() {
        let t = i18n.context('app.file')
        let hash = this.generateTreeHash()

        if (hash !== this.treeHash) {
            let answer = dialog.showMessageBox(
                t('Your changes will be lost if you close this file without saving. Do you want to continue?'),
                'warning',
                [t('Save'), t('Don’t Save'), t('Cancel')], 2
            )

            if (answer === 0) return true
            else if (answer === 2) return false
        }

        return true
    }

    // Playing

    clickVertex(vertex, {button = 0} = {}) {
        this.closeDrawer()

        let {gameTrees, gameIndex, gameCurrents, treePosition} = this.state
        let tree = gameTrees[gameIndex]
        let board = gametree.getBoard(tree, treePosition)

        if (typeof vertex == 'string') {
            vertex = board.coord2vertex(vertex)
        }

        if (['play', 'autoplay'].includes(this.state.mode)) {
            if (button === 0) {
                if (board.get(vertex) === 0) {
                    let autoGenmove = setting.get('gtp.auto_genmove')
                    // BUGOUT safety: check that we're allowed to move,
                    // and not accidentally bouncing the finger
                    // and sending some additional move as the opponent
                    let color = this.inferredState.currentPlayer > 0 ? 'B' : 'W'

                    let multiplayerColorSatisfied = this.state.multiplayer.yourColor && this.state.multiplayer.yourColor.event && this.state.multiplayer.yourColor.event.yourColor && color === this.state.multiplayer.yourColor.event.yourColor[0]
                    
                    let botColorSatisfied = this.state.multiplayer.botColor && this.state.multiplayer.botColor[0] !== color
                    if (this.state.multiplayer && (multiplayerColorSatisfied || botColorSatisfied)) {
                        this.makeMove(vertex, { sendToEngine: autoGenmove })
                    }
                }
            }
        } else if (['scoring', 'estimator'].includes(this.state.mode)) {
            if (button !== 0 || board.get(vertex) === 0) return

            let {mode, deadStones} = this.state
            let dead = deadStones.some(v => helper.vertexEquals(v, vertex))
            let stones = mode === 'estimator' ? board.getChain(vertex) : board.getRelatedChains(vertex)

            if (!dead) {
                deadStones = [...deadStones, ...stones]
            } else {
                deadStones = deadStones.filter(v => !stones.some(w => helper.vertexEquals(v, w)))
            }

            this.setState({deadStones})
        }

        this.events.emit('vertexClick')
    }

    makeMove(vertex, {player = null, sendToEngine = false} = {}) {
        if (!['play'].includes(this.state.mode)) {
            this.closeDrawer()
            this.setMode('play')
        }

        let t = i18n.context('app.play')
        let {gameTrees, gameIndex, treePosition} = this.state
        let tree = gameTrees[gameIndex]
        let node = tree.get(treePosition)
        let board = gametree.getBoard(tree, treePosition)

        if (typeof vertex == 'string') {
            vertex = board.coord2vertex(vertex)
        }

        let pass = !board.hasVertex(vertex)
        if (!pass && board.get(vertex) !== 0) return

        let prev = tree.get(node.parentId)
        if (!player) player = this.inferredState.currentPlayer
        let color = player > 0 ? 'B' : 'W'
        let capture = false, suicide = false, ko = false
        let newNodeData = {[color]: [sgf.stringifyVertex(vertex)]}

        if (!pass) {
            // Check for ko

            if (prev != null && setting.get('game.show_ko_warning')) {
                let hash = board.makeMove(player, vertex).getPositionHash()
                let prevBoard = gametree.getBoard(tree, prev.id)

                ko = prevBoard.getPositionHash() === hash

                if (ko && dialog.showMessageBox(
                    t([
                        'You are about to play a move which repeats a previous board position.',
                        'This is invalid in some rulesets. Do you want to play anyway?'
                    ].join('\n')),
                    'info',
                    [t('Play Anyway'), t('Don’t Play')], 1
                ) != 0) return
            }

            let vertexNeighbors = board.getNeighbors(vertex)

            // Check for suicide

            capture = vertexNeighbors
                .some(v => board.get(v) == -player && board.getLiberties(v).length == 1)

            suicide = !capture
            && vertexNeighbors.filter(v => board.get(v) == player)
                .every(v => board.getLiberties(v).length == 1)
            && vertexNeighbors.filter(v => board.get(v) == 0).length == 0

            if (suicide && setting.get('game.show_suicide_warning')) {
                if (dialog.showMessageBox(
                    t([
                        'You are about to play a suicide move.',
                        'This is invalid in some rulesets. Do you want to play anyway?'
                    ].join('\n')),
                    'info',
                    [t('Play Anyway'), t('Don’t Play')], 1
                ) != 0) return
            }
        }

        // Update data

        let nextTreePosition
        let newTree = tree.mutate(draft => {
            nextTreePosition = draft.appendNode(treePosition, newNodeData)
        })

        let createNode = tree.get(nextTreePosition) == null

        this.setCurrentTreePosition(newTree, nextTreePosition)

        // Play sounds

        if (!pass) {
            let delay = setting.get('sound.capture_delay_min')
            delay += Math.floor(Math.random() * (setting.get('sound.capture_delay_max') - delay))

            if (capture || suicide) sound.playCapture(delay)
            sound.playPachi()
        } else {
            sound.playPass()
        }

        // Enter scoring mode after two consecutive passes

        let enterScoring = false

        if (pass && createNode && prev != null) {
            let prevColor = color === 'B' ? 'W' : 'B'
            let prevPass = node.data[prevColor] != null && node.data[prevColor][0] === ''

            if (prevPass) {
                this.events.emit('bugout-consecutive-pass')
                enterScoring = true
                this.setMode('scoring')
            }
        }

        // Emit event

        this.events.emit('moveMake', {pass, capture, suicide, ko, enterScoring})

        if (sendToEngine && this.attachedEngineSyncers.some(x => x != null)) {
            // Send command to engine

            let passPlayer = pass ? player : null
            setTimeout(() => this.generateMove({passPlayer}), setting.get('gtp.move_delay'))
        }
    }

    makeResign({player = null} = {}) {
        let {gameTrees, gameIndex} = this.state
        let {currentPlayer} = this.inferredState
        if (player == null) player = currentPlayer
        let color = player > 0 ? 'W' : 'B'
        let tree = gameTrees[gameIndex]

        this.makeMove([-1, -1], {player})

        this.events.emit('resign', {player})
    }

    // Navigation

    setCurrentTreePosition(tree, id, {clearCache = false} = {}) {
        if (clearCache) gametree.clearBoardCache()

        if (['scoring', 'estimator'].includes(this.state.mode)) {
            this.setState({mode: 'play'})
        }

        let {gameTrees, gameCurrents} = this.state
        let gameIndex = gameTrees.findIndex(t => t.root.id === tree.root.id)
        let currents = gameCurrents[gameIndex]

        let n = tree.get(id)
        while (n.parentId != null) {
            // Update currents

            currents[n.parentId] = n.id
            n = tree.get(n.parentId)
        }

        let prevGameIndex = this.state.gameIndex
        let prevTreePosition = this.state.treePosition

        this.setState({
            playVariation: null,
            blockedGuesses: [],
            highlightVertices: [],
            gameTrees: gameTrees.map((t, i) => i !== gameIndex ? t : tree),
            gameIndex,
            treePosition: id
        })

        this.recordHistory({prevGameIndex, prevTreePosition})

        this.events.emit('navigate')
    }
    
    // 😇 BUGOUT trimmed 😇
    

    // Node Actions

    getGameInfo(tree) {
        let komi = gametree.getRootProperty(tree, 'KM')
        if (komi != null && !isNaN(komi)) komi = +komi
        else komi = null

        let size = gametree.getRootProperty(tree, 'SZ')
        if (size == null) {
            size = [19, 19]
        } else {
            let s = size.toString().split(':')
            size = [+s[0], +s[s.length - 1]]
        }

        let handicap = gametree.getRootProperty(tree, 'HA', 0)
        handicap = Math.max(1, Math.min(9, Math.round(handicap)))
        if (handicap === 1) handicap = 0

        let playerNames = ['B', 'W'].map(x =>
            gametree.getRootProperty(tree, `P${x}`) || gametree.getRootProperty(tree, `${x}T`)
        )

        let playerRanks = ['BR', 'WR'].map(x => gametree.getRootProperty(tree, x))

        return {
            playerNames,
            playerRanks,
            blackName: playerNames[0],
            blackRank: playerRanks[0],
            whiteName: playerNames[1],
            whiteRank: playerRanks[1],
            gameName: gametree.getRootProperty(tree, 'GN'),
            eventName: gametree.getRootProperty(tree, 'EV'),
            gameComment: gametree.getRootProperty(tree, 'GC'),
            date: gametree.getRootProperty(tree, 'DT'),
            result: gametree.getRootProperty(tree, 'RE'),
            komi,
            handicap,
            size
        }
    }

    // BUGOUT calls this
    setGameInfo(tree, data) {
        let newTree = tree.mutate(draft => {
            if ('size' in data) {
                // Update board size

                if (data.size) {
                    let value = data.size
                    value = value.map(x => isNaN(x) || !x ? 19 : Math.min(25, Math.max(2, x)))

                    if (value[0] === value[1]) value = value[0].toString()
                    else value = value.join(':')

                    //
                    // BUGOUT: do not update default setting
                    //
                    
                    draft.updateProperty(draft.root.id, 'SZ', [value])
                } else {
                    draft.removeProperty(draft.root.id, 'SZ')
                }
            }
        })

        newTree = newTree.mutate(draft => {
            let props = {
                blackName: 'PB',
                blackRank: 'BR',
                whiteName: 'PW',
                whiteRank: 'WR',
                gameName: 'GN',
                eventName: 'EV',
                gameComment: 'GC',
                date: 'DT',
                result: 'RE',
                komi: 'KM',
                handicap: 'HA'
            }

            for (let key in props) {
                if (!(key in data)) continue
                let value = data[key]

                if (value && value.toString() !== '') {
                    if (key === 'komi') {
                        if (isNaN(value)) value = 0

                        setting.set('game.default_komi', value)
                    } else if (key === 'handicap') {
                        let board = gametree.getBoard(newTree, newTree.root.id)
                        let stones = board.getHandicapPlacement(+value)

                        value = stones.length
                        setting.set('game.default_handicap', value)

                        if (value <= 1) {
                            draft.removeProperty(draft.root.id, props[key])
                            draft.removeProperty(draft.root.id, 'AB')
                            continue
                        }

                        draft.updateProperty(draft.root.id, 'AB', stones.map(sgf.stringifyVertex))
                    }

                    draft.updateProperty(draft.root.id, props[key], [value.toString()])
                } else {
                    draft.removeProperty(draft.root.id, props[key])
                }
            }
        })

        this.setCurrentTreePosition(newTree, this.state.treePosition)
    }

    getPlayer(tree, treePosition) {
        let {data} = tree.get(treePosition)

        return data.PL != null ? (data.PL[0] === 'W' ? -1 : 1)
            : data.B != null || data.HA != null && +data.HA[0] >= 1 ? -1
            : 1
    }

    setPlayer(tree, treePosition, sign) {
        let newTree = tree.mutate(draft => {
            let node = draft.get(treePosition)
            let intendedSign = node.data.B != null || node.data.HA != null
                && +node.data.HA[0] >= 1 ? -1 : +(node.data.W != null)

            if (intendedSign === sign || sign === 0) {
                draft.removeProperty(treePosition, 'PL')
            } else {
                draft.updateProperty(treePosition, 'PL', [sign > 0 ? 'B' : 'W'])
            }
        })

        this.setCurrentTreePosition(newTree, treePosition)
    }

    // GTP Engines

    attachEngines(...engines) {
        let {attachedEngines} = this.state

        if (helper.vertexEquals([...engines].reverse(), attachedEngines)) {
            // Just swap engines

            this.attachedEngineSyncers.reverse()

            this.setState(({engineBusy, engineCommands}) => ({
                engineCommands: engineCommands.reverse(),
                engineBusy: engineBusy.reverse(),
                attachedEngines: engines
            }))

            return
        }

        let quitTimeout = setting.get('gtp.engine_quit_timeout')

        for (let i = 0; i < attachedEngines.length; i++) {
            if (attachedEngines[i] === engines[i]) continue

            if (this.attachedEngineSyncers[i]) {
                this.attachedEngineSyncers[i].controller.stop(quitTimeout)
            }

            try {
                let engine = engines[i]

                let syncer = new EngineSyncer(engine, 
                    {
                        entryMethod: this.state.multiplayer && this.state.multiplayer.entryMethod,
                        joinPrivateGame: this.bugout.joinPrivateGame,
                        handleWaitForOpponent: data => {
                            this.setState({
                                multiplayer: {
                                    ...this.state.multiplayer,
                                    waitForOpponentModal: data
                                }
                            })
                        },
                        handleYourColor: data => {
                            this.setState({
                                multiplayer: {
                                    ...this.state.multiplayer,
                                    yourColor: data
                                }
                            })
                        }
                    }) // 😇BUGOUT😇
                this.attachedEngineSyncers[i] = syncer

                syncer.on('busy-changed', () => {
                    this.setState(({engineBusy}) => {
                        let j = this.attachedEngineSyncers.indexOf(syncer)
                        engineBusy[j] = syncer.busy

                        return {engineBusy}
                    })
                })

                syncer.controller.on('command-sent', evt => {
                    if (evt.command.name === 'list_commands') {
                        evt.getResponse().then(response =>
                            this.setState(({engineCommands}) => {
                                let j = this.attachedEngineSyncers.indexOf(syncer)
                                engineCommands[j] = response.content.split('\n')

                                return {engineCommands}
                            })
                        ).catch(helper.noop)
                    }

                    this.handleCommandSent(Object.assign({syncer}, evt))
                })


                syncer.controller.start()
            } catch (err) {
                this.attachedEngineSyncers[i] = null
                engines[i] = null
            }
        }

        this.setState({attachedEngines: engines})
    }

    detachEngines() {
        this.attachEngines(null, null)
    }

    suspendEngines() {
        for (let syncer of this.attachedEngineSyncers) {
            if (syncer != null) {
                syncer.controller.kill()
            }
        }

        this.stopGeneratingMoves()
        this.setBusy(false)
    }

    handleCommandSent({syncer, command, subscribe, getResponse}) {
        let sign = 1 - this.attachedEngineSyncers.indexOf(syncer) * 2
        if (sign > 1) sign = 0

        let t = i18n.context('app.engine')
        let entry = {sign, name: syncer.engine.name, command, waiting: true}
        
        let updateEntry = update => {
            Object.assign(entry, update)
        }

        subscribe(({response, end}) => {
            updateEntry({
                response: Object.assign({}, response),
                waiting: !end
            })
        })

        getResponse()
        .catch(_ => {
            updateEntry({
                response: {internal: true, content: t('connection failed')},
                waiting: false
            })
        })
    }

    async syncEngines({showErrorDialog = false} = {}) {
        if (this.attachedEngineSyncers.every(x => x == null)) return
        if (this.engineBusySyncing) return

        let t = i18n.context('app.engine')
        this.engineBusySyncing = true

        try {
            while (true) {
                let {gameTrees, gameIndex, treePosition} = this.state
                let tree = gameTrees[gameIndex]

                await Promise.all(this.attachedEngineSyncers.map(syncer => {
                    if (syncer == null) return
                    return syncer.sync(tree, treePosition)
                }))

                if (treePosition === this.state.treePosition) break
            }
        } catch (err) {
            this.engineBusySyncing = false

            if (showErrorDialog) {
                dialog.showMessageBox(t(err.message), 'warning')
            } else {
                throw err
            }
        }

        this.engineBusySyncing = false
    }

    async generateMove({firstMove = true, followUp = false} = {}) {
        this.closeDrawer()

        if (!firstMove && !this.state.generatingMoves) {
            return
        } else if (firstMove) {
            this.setState({generatingMoves: true})
        }

        let t = i18n.context('app.engine')
        let {gameTrees, gameIndex} = this.state
        let {currentPlayer} = this.inferredState
        let tree = gameTrees[gameIndex]
        let [color, opponent] = currentPlayer > 0 ? ['B', 'W'] : ['W', 'B']
        let [playerIndex, otherIndex] = currentPlayer > 0 ? [0, 1] : [1, 0]
        let playerSyncer = this.attachedEngineSyncers[playerIndex]
        let otherSyncer = this.attachedEngineSyncers[otherIndex]

        if (playerSyncer == null) {
            if (otherSyncer != null) {
                // Switch engines, so the attached engine can play

                let engines = [...this.state.attachedEngines].reverse()
                this.attachEngines(...engines)
                ;[playerSyncer, otherSyncer] = [otherSyncer, playerSyncer]
            } else {
                return
            }
        }

        this.setBusy(true)

        try {
            await this.syncEngines({showErrorDialog: false})
        } catch (err) {
            this.stopGeneratingMoves()
            this.setBusy(false)

            return
        }

        let {commands} = this.attachedEngineSyncers[playerIndex]
        let commandName = ['genmove_analyze', 'lz-genmove_analyze', 'genmove'].find(x => commands.includes(x))
        if (commandName == null) commandName = 'genmove'

        let responseContent = await (
            commandName === 'genmove'
            ? playerSyncer.controller.sendCommand({name: commandName, args: [color]})
                .then(res => res.content)
            : new Promise((resolve, reject) => {
                let interval = setting.get('board.analysis_interval').toString()

                playerSyncer.controller.sendCommand(
                    {name: commandName, args: [color, interval]},
                    ({line}) => {
                        if (line.indexOf('play ') !== 0) return
                        resolve(line.slice('play '.length).trim())
                    }
                )
                .then(() => resolve(null))
                .catch(reject)
            })
        ).catch(() => null)

        let sign = color === 'B' ? 1 : -1
        let pass = true
        let vertex = [-1, -1]
        let board = gametree.getBoard(tree, tree.root.id)

        if (responseContent == null) {
            this.stopGeneratingMoves()
            this.setBusy(false)

            return
        } else if (responseContent.toLowerCase() !== 'pass') {
            pass = false

            if (responseContent.toLowerCase() === 'resign') {
                dialog.showMessageBox(t(p => `${p.engineName} has resigned.`, {
                    engineName: playerSyncer.engine.name
                }))

                this.stopGeneratingMoves()
                this.makeResign()
                this.setBusy(false)

                return
            }

            vertex = board.coord2vertex(responseContent)
        }

        let previousNode = tree.get(this.state.treePosition)
        let previousPass = previousNode != null && ['W', 'B'].some(color =>
            previousNode.data[color] != null
            && !board.hasVertex(sgf.parseVertex(previousNode.data[color][0]))
        )
        let doublePass = previousPass && pass

        this.makeMove(vertex, {player: sign})

        if (followUp && otherSyncer != null && !doublePass) {
            await helper.wait(setting.get('gtp.move_delay'))
            this.generateMove({passPlayer: pass ? sign : null, firstMove: false, followUp})
        } else {
            this.stopGeneratingMoves()
        }

        this.setBusy(false)
    }

    stopGeneratingMoves() {
        if (!this.state.generatingMoves) return

        let t = i18n.context('app.engine')

        this.setState({generatingMoves: false})
    }

    // Render

    render(_, state) {
        // Calculate some inferred values

        let {gameTrees, gameIndex, treePosition} = state
        let tree = gameTrees[gameIndex]
        let scoreBoard, areaMap

        if (['scoring', 'estimator'].includes(state.mode)) {
            // Calculate area map

            scoreBoard = gametree.getBoard(tree, state.treePosition).clone()

            for (let vertex of state.deadStones) {
                let sign = scoreBoard.get(vertex)
                if (sign === 0) continue

                scoreBoard.captures[sign > 0 ? 1 : 0]++
                scoreBoard.set(vertex, 0)
            }

            areaMap = state.mode === 'estimator'
                ? influence.map(scoreBoard.arrangement, {discrete: true})
                : influence.areaMap(scoreBoard.arrangement)
        }

        this.inferredState = {
            gameTree: tree,
            gameInfo: this.getGameInfo(tree),
            currentPlayer: this.getPlayer(tree, treePosition),
            scoreBoard,
            areaMap
        }

        state = Object.assign(state, this.inferredState)

        this.bugout.enterGame(this, state)   // 😀
        this.bugout.announceTurn(tree, treePosition, this.events) // 😀

        return h('section',
            {
                class: classNames({
                    leftsidebar: state.showLeftSidebar,
                    sidebar: state.showSidebar,
                    [state.mode]: true
                })
            },

            // ↓ BUGOUT ↓
            h(GameLobbyModal, {
                joinPrivateGame: this.bugout.joinPrivateGame.join,
                idleStatus: state.multiplayer && state.multiplayer.idleStatus && state.multiplayer.idleStatus.status,
                update: entryMethod => this.setState({ 
                    multiplayer: {
                        ...this.state.multiplayer,
                        entryMethod
                    }
                }),
                appEvents: this.events
            }),
            h(WaitForOpponentModal, {
                data: state.multiplayer && state.multiplayer.waitForOpponentModal,
                reconnectDialog: state.multiplayer && state.multiplayer.reconnectDialog
            }),
            h(MultiplayerColorPrefModal, {
                data: state.multiplayer,
                idleStatus: state.multiplayer && state.multiplayer.idleStatus && state.multiplayer.idleStatus.status
            }),
            h(PlayBotColorSelectionModal, {
                data: state.multiplayer
            }),
            h(BoardSizeModal, {
                data: state.multiplayer,
                chooseBoardSize: boardSize => { 
                    this.setState({
                        multiplayer: {
                            ...this.state.multiplayer,
                            boardSize
                        }
                    })
                    this.events.emit('choose-board-size', { boardSize })
                }
            }),
            h(WaitForYourColorModal, {
                data: state.multiplayer
            }),
            h(YourColorChosenModal, { yourColor: state.multiplayer && state.multiplayer.yourColor }), 
            h(ReconnectModal, { data: state.multiplayer }), 
            h(IdleStatusModal, { data: state.multiplayer }),
            h(OpponentPassedModal),
            h(OpponentQuitModal),
            h(WaitForBotModal),
            // ↑ BUGOUT ↑

            h(MainView, state),
            h(DrawerManager, state)
        )
    }
}

// Render

render(h(App), document.body)
