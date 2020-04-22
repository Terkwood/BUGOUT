# Sabaki Object

`sabaki` is a global object, giving users access to the Sabaki API.

## Events

To listen to events, use the [`EventEmitter`](https://nodejs.org/api/events.html#events_class_eventemitter) `sabaki.events` like this:

~~~js
sabaki.events.on('ready', () => {
    console.log('Preparation complete!')
})
~~~

### Event: 'ready'

The `ready` event is emitted after the page is ready, Sabaki has loaded all settings, and all components are ready to use.

### Event: 'modeChange'

The `modeChange` event is emitted after Sabaki changes its mode.

### Event: 'navigate'

The `navigate` event is emitted when Sabaki has finished loading a game tree position.

### Event: 'vertexClick'

* `evt` `<Object>`
    * `vertex` [`<Vertex>`](vertex.md)
    * `options` `<Object>`
        * `button` `<Integer>`
        * `ctrlKey` `<Boolean>`
        * `x` `<Integer>`
        * `y` `<Integer>`

The `vertexClick` event is emitted when the user clicks on the board.

### Event: 'moveMake'

* `evt` `<Object>`
    * `pass` `<Boolean>` - Specifies whether the move was a pass
    * `capture` `<Boolean>` - Specifies whether the move has captured some stones
    * `suicide` `<Boolean>` - Specifies whether the move was a suicide
    * `ko` `<Boolean>` - Specifies whether the move violates the simple ko rule

The `moveMake` event is emitted after a move has been played, either a stone has been placed or a pass has been made.

### Event: 'resign'

* `player` [`<Sign>`](sign.md)

The `resign` event is triggered after someone resigns.

### Event: 'toolUse'

* `evt` `<Object>`
    * `tool` `<String>`
    * `vertex` [`<Vertex>`](vertex.md)
    * `argument` [`<Vertex>`](vertex.md)

The `toolUse` event is triggered after the user used `tool` by clicking on `vertex`. `tool` can be one of the following: `'stone_1'`, `'stone_'-1`, `'cross'`, `'triangle'`, `'square'`, `'circle'`, `'line'`, `'arrow'`, `'label'`, `'number'`. If `tool` is `'line'` or `'arrow'`, `argument` is the end vertex. If `tool` is `'label'` or `'number'`, `argument` is the label text.

### Event: 'fileLoad'

The `fileLoad` event is triggered when Sabaki finishes loading some file.

## Properties

* `events` [`<EventEmitter>`](https://nodejs.org/api/events.html)
* `appName` `<String>`
* `version` `<String>`
* `window` [`<BrowserWindow>`](http://electron.atom.io/docs/api/browser-window/)

## State

State properties can be accessed through the object `sabaki.state`. Generally, *do not* change state directly through `sabaki.setState()`, or worse, by mutating `sabaki.state`.

* `mode` `<String>` - One of `'play'`, `'edit'`, `'find'`, `'scoring'`, `'estimator'`, `'guess'`, `'autoplay'`
* `openDrawer` `<String>` | `<Null>` - One of `'info'`, `'gamechooser'`, `'cleanmarkup'`, `'score'`, `'preferences'` if a drawer is open, otherwise `null`
* `busy` `<Number>` - Sabaki is busy if and only if the integer is positive
* `fullScreen` `<Boolean>`
* `representedFilename` `<String>` | `<Null>`
* `gameTrees` [`<GameTree[]>`](gametree.md)
* `treePosition` `<TreePosition>`
* `undoable` `<Boolean>`
* `selectedTool` `<String>` - One of `'stone_1'`, `'stone_'-1`, `'cross'`, `'triangle'`, `'square'`, `'circle'`, `'line'`, `'arrow'`, `'label'`, `'number'`
* `scoringMethod` `<String>` - One of `'territory'`, `'area'`
* `findText` `<String>`
* `findVertex` [`<Vertex>`](vertex.md) | `<Null>`
* `attachedEngines` `<Object[]>`
* `engineCommands` `<String[][]>`
* `generatingMoves` `<Boolean>`

## Methods

### User Interface

#### sabaki.setMode(mode)

* `mode` `<String>` - One of `'play'`, `'edit'`, `'find'`, `'scoring'`, `'estimator'`, `'guess'`, `'autoplay'`

#### sabaki.openDrawer(drawer)

* `drawer` `<String>` - One of `'info'`, `'gamechooser'`, `'cleanmarkup'`, `'score'`, `'preferences'`

The score drawer should only be opened in scoring mode or estimator mode.

#### sabaki.closeDrawer()

#### sabaki.setBusy(busy)

* `busy` `<Boolean>`

Set `busy` to `true` to indicate to the user that Sabaki is busy doing stuff. The user cannot interact with the UI in busy state. Don't forget to call `setBusy(false)` after `setBusy(true)`.

### File Management

#### sabaki.getEmptyGameTree()

Returns an empty [game tree](gametree.md) with the default board size, komi, and handicap settings.

#### async sabaki.newFile([options])

* `options` `<Object>` *(optional)*
    * `playSound` `<Boolean>` *(optional)* - Default: `false`
    * `showInfo` `<Boolean>` *(optional)* - Default: `false`
    * `suppressAskForSave` `<Boolean>` *(optional)* - Default: `false`

Resets file name, returns to play mode, and replaces current file with an empty file. Set `showInfo` to `true` if you want the 'Game Info' drawer to show afterwards.

If there's a modified file opened, Sabaki will ask the user to save the file first depending whether `suppressAskForSave` is `false`. Set `suppressAskForSave` to `true` to suppress this question.

#### async sabaki.loadContent(content, extension[, options])

* `content` `<String>`
* `extension` `<String>` - File extension, e.g. `'sgf'`
* `options` `<Object>` *(optional)*
    * `suppressAskForSave` `<Boolean>` *(optional)* - Default: `false`

Returns to play mode and parses `content` which replaces current file. Sabaki will automatically detect file format by `extension`.

If there's a modified file opened, Sabaki will ask the user to save the file first depending whether `suppressAskForSave` is `false`. Set `suppressAskForSave` to `true` to suppress this question.

Returns `true` if the operation succeeded, otherwise `false`.

#### sabaki.saveFile([filename])

* `filename` `<String>` *(optional)*

Saves current file in given `filename` as SGF. If `filename` is not set, Sabaki will show a save file dialog. On the web version `filename` is ignored and treated as if not set.

#### sabaki.getSGF()

Returns the SGF of the current file as a string.

#### sabaki.askForSave()

If there's a modified file opened, Sabaki will ask the user to save the file first or to cancel the action. Returns `true` if the user saved the file or wants to proceed without saving, and `false` if the user wants to cancel the action.

### Playing

#### sabaki.clickVertex(vertex[, options])

* `vertex` [`<Vertex>`](vertex.md)
* `options` `<Object>` *(optional)*
    * `button` `<Integer>` *(optional)* - Default: `0`
    * `ctrlKey` `<Boolean>` *(optional)* - Default: `false`
    * `x` `<Boolean>` *(optional)* - Default: `0`
    * `y` `<Boolean>` *(optional)* - Default: `0`

Performs a click on the given vertex position on the board with given button index, whether the control key is pressed, and the mouse position. The mouse position is only needed for displaying the context menu.

#### sabaki.makeMove(vertex[, options])

* `vertex` [`<Vertex>`](vertex.md)
* `options` `<Object>` *(optional)*
    * `player` [`<Sign>`](sign.md) *(optional)* - Default: Current player
    * `clearUndoPoint` `<Boolean>` *(optional)* - Default: `true`
    * `sendToEngine` `<Boolean>` *(optional)* - Default: `false`

Makes a proper move on the given vertex on the current board as given `player`. If `vertex` is not on the board or `player` equals `0`, Sabaki will make a pass instead.

Depending on the settings, Sabaki may notify the user about ko and suicide, plays a sound.

#### sabaki.makeResign()

Updates game information that the current player has resigned and shows the game info drawer for the user.

### Navigation

#### sabaki.setCurrentTreePosition(tree, treePosition[, options])

* `tree` [`<GameTree>`](gametree.md)
* `treePosition` `<TreePosition>`
* `options` `<Object>` *(optional)*
    * `clearCache` `<Boolean>` - Default: `false`

Updates the tree and jumps to the given `treePosition`. Make sure the root id of the given `tree` coincides with a tree in the `gameTrees` state.

### Node Actions

#### sabaki.getGameInfo(tree)

* `tree` [`<GameTree>`](gametree.md)

Returns an object with the following values:

* `blackName` `<String>` | `<Null>`
* `blackRank` `<String>` | `<Null>`
* `whiteName` `<String>` | `<Null>`
* `whiteRank` `<String>` | `<Null>`
* `gameName` `<String>` | `<Null>`
* `eventName` `<String>` | `<Null>`
* `date` `<String>` | `<Null>`
* `result` `<String>` | `<Null>`
* `komi` `<Float>` | `<Null>`
* `handicap` `<Integer>`
* `size` `<Integer[]>` - An array of two numbers, representing the width and height of the game board

#### sabaki.setGameInfo(tree, data)

* `tree` [`<GameTree>`](gametree.md)
* `data` `<Object>`
    * `blackName` `<String>` | `<Null>` *(optional)*
    * `blackRank` `<String>` | `<Null>` *(optional)*
    * `whiteName` `<String>` | `<Null>` *(optional)*
    * `whiteRank` `<String>` | `<Null>` *(optional)*
    * `gameName` `<String>` | `<Null>` *(optional)*
    * `eventName` `<String>` | `<Null>` *(optional)*
    * `date` `<String>` | `<Null>` *(optional)*
    * `result` `<String>` | `<Null>` *(optional)*
    * `komi` `<Float>` | `<Null>` *(optional)*
    * `handicap` `<Integer>` | `<Null>` *(optional)*
    * `size` `<Integer[]>` | `<Null>` *(optional)* - An array of two numbers, representing the width and height of the game board

Don't provide keys in `data` to leave corresponding information unchanged in the game tree. Set corresponding keys in `data` to `null` to remove the data from the game tree.

#### sabaki.getPlayer(tree, treePosition)

* `tree` [`<GameTree>`](gametree.md)
* `treePosition` `<TreePosition>`

Returns a [sign](sign.md) corresponding to the player that should be playing at the given `treePosition`.

#### sabaki.setPlayer(tree, treePosition, sign)

* `tree` [`<GameTree>`](gametree.md)
* `treePosition` `<TreePosition>`
* `sign` [`<Sign>`](sign.md) - Cannot be `0`
