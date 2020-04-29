# Building & Tests

## Building

Building BUGOUT browser requires [Node.js 6.2.x or later](https://nodejs.org/en/download/) and npm. First, clone the repo:

~~~
$ git clone https://github.com/Terkwood/BUGOUT
$ cd BUGOUT/browser
~~~

### Desktop version

Install the dependencies of the browser using npm:

~~~
$ npm install
~~~

browser uses webpack to bundle all files into one single file. For development use the following command to create bundles automatically while you edit files:

~~~
$ npm run watch
~~~

To build a minified version:

~~~
npm run build
~~~

This creates a `bundle.js` file. To run Sabaki, simply open `browser/index.html` in a modern web browser, preferably Chrome.
