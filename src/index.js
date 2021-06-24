const { program } = require("commander");
const Jimp = require("jimp");
const fs = require("fs");
const logger = require("./log");
const util = require("./util");
program
  .version("0.0.1")
  .arguments("<image>")
  .description("pic2bombhopper", {
    image: "Path to image",
  })
  .option("--debug", "Enables Debug")
  .option("-o, --output <path>", "Output path", "level.json")
  .option("-g, --grid-size <grid-size>", "Grid Size of pixels", 20)
  .action((image, options) => {
    if (options.debug) logger.info(options);
    if (options.gridSize)
      logger.info(`Building with a grid size of ${options.gridSize}`);
    Jimp.read(image)
      .then((image) => {
        logger.info("Successfully read image.");
        const { width, height } = image.bitmap;
        const level = {
          name: "Generated with pic2bombhopper",
          timings: [0, 0],
          entities: [
            {
              type: "player",
              params: {
                isStatic: true,
                angle: 0,
                x: 0,
                y: 0,
                magazine: ["empty"],
              },
            },
          ],
          formatVersion: 0,
        };
        if (width * height >= 2048) {
          logger.warn(
            `This tool was created for pixel art. Using bigger images may lag the game and create large files. This image will create ${
              width * height
            } objects.`
          );
        }
        image.scan(0, 0, width, height, (x, y, idx) => {
          const red = image.bitmap.data[idx + 0];
          const green = image.bitmap.data[idx + 1];
          const blue = image.bitmap.data[idx + 2];
          const alpha = image.bitmap.data[idx + 3];
          if (alpha === 0) {
            return;
          } else if (alpha <= 254) {
            logger.warn(
              "Bombhopper currently does not support partial transparency, will default to no transparency"
            );
          }
          level.entities.push({
            type: "paint",
            params: {
              fillColor: util.rgbToDec(red, green, blue),
              vertices: [
                {
                  x: x * options.gridSize,
                  y: y * options.gridSize,
                },
                {
                  x: x * options.gridSize + options.gridSize,
                  y: y * options.gridSize,
                },
                {
                  x: x * options.gridSize + options.gridSize,
                  y: y * options.gridSize + options.gridSize,
                },
                {
                  x: x * options.gridSize,
                  y: y * options.gridSize + options.gridSize,
                },
              ],
            },
          });
        });
        logger.info(`Writing to ${options.output}`);
        fs.writeFileSync(options.output, JSON.stringify(level));
      })
      .catch((err) => {
        logger.error(err);
      });
  })
  .parse(process.argv);
