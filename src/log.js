const chalk = require("chalk");
class Logger {
  static error(text) {
    console.log(`${chalk.bgRed.bold(" ERROR ")}   ${text}`);
  }
  static warn(text) {
    console.log(`${chalk.bgYellow.black.bold(" WARN ")}   ${text}`);
  }
  static info(text) {
    console.log(`${chalk.bgBlue.bold(" INFO ")}   ${text}`);
  }
}
module.exports = Logger;
