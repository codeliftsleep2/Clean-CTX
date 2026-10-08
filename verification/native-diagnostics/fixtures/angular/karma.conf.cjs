module.exports = function (config) {
  config.set({
    frameworks: ['jasmine'],
    plugins: [require('karma-jasmine'), require('karma-chrome-launcher'), require('karma-coverage')],
    customLaunchers: {
      ChromeHeadlessOperator: { base: 'ChromeHeadless', flags: ['--no-sandbox', '--disable-dev-shm-usage'] }
    }
  });
};
