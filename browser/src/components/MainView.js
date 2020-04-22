const {h, Component} = require('preact')

const Goban = require('./Goban')
const PlayBar = require('./bars/PlayBar')
const ScoringBar = require('./bars/ScoringBar')

const gametree = require('../modules/gametree')

class MainView extends Component {
    constructor(props) {
        super(props)

        this.handleGobanVertexClick = this.handleGobanVertexClick.bind(this)
    }

    handleGobanVertexClick(evt) {
        sabaki.clickVertex(evt.vertex, evt)
    }


    render({
        mode,
        gameTree,
        treePosition,
        currentPlayer,
        gameInfo,
        attachedEngines,
        engineBusy,
        analysisTreePosition,

        deadStones,
        scoringMethod,
        scoreBoard,
        playVariation,
        analysis,
        areaMap,

        showCoordinates,
        showMoveColorization,
        showMoveNumbers,
        showNextMoves,
        showSiblings,
        fuzzyStonePlacement,
        animateStonePlacement,

        showLeftSidebar,
        showSidebar,
        sidebarWidth,
        leftSidebarWidth
    }, {
        gobanCrosshair
    }) {
        let node = gameTree.get(treePosition)
        let board = gametree.getBoard(gameTree, treePosition)
        let komi = +gametree.getRootProperty(gameTree, 'KM', 0)
        let handicap = +gametree.getRootProperty(gameTree, 'HA', 0)
        let paintMap

        if (['scoring', 'estimator'].includes(mode)) {
            paintMap = areaMap
        }

        return h('section',
            {
                id: 'main',
                style: {
                    left: showLeftSidebar ? leftSidebarWidth : null,
                    right: showSidebar ? sidebarWidth : null
                }
            },

            h('main',
                {ref: el => this.mainElement = el},

                h(Goban, {
                    gameTree,
                    treePosition,
                    board,
                    highlightVertices: false,
                    analysis: mode === 'play'
                        && analysisTreePosition != null
                        && analysisTreePosition === treePosition
                        ? analysis
                        : null,
                    paintMap,
                    dimmedStones: ['scoring', 'estimator'].includes(mode) ? deadStones : [],

                    crosshair: gobanCrosshair,
                    showCoordinates,
                    showMoveColorization,
                    showMoveNumbers,
                    showNextMoves,
                    showSiblings,
                    fuzzyStonePlacement,
                    animateStonePlacement,

                    playVariation,
                    drawLineMode: null,

                    onVertexClick: this.handleGobanVertexClick
                })
            ),

            h('section', {id: 'bar'},
                h(PlayBar, {
                    mode,
                    attachedEngines,
                    playerBusy: engineBusy,
                    playerNames: gameInfo.playerNames,
                    playerRanks: gameInfo.playerRanks,
                    playerCaptures: board.captures,
                    currentPlayer,
                    showHotspot: node.data.HO != null
                }),

                h(ScoringBar, {
                    type: 'scoring',
                    mode,
                    method: scoringMethod,
                    scoreBoard,
                    areaMap,
                    komi,
                    handicap
                }),

                h(ScoringBar, {
                    type: 'estimator',
                    mode,
                    method: scoringMethod,
                    scoreBoard,
                    areaMap,
                    komi,
                    handicap
                })
            )
        )
    }
}

module.exports = MainView
