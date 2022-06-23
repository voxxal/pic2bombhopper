const decToHex = (value) => {
  if (value > 255) {
    return "FF";
  } else if (value < 0) {
    return "00";
  } else {
    return value.toString(16).padStart(2, "0");
  }
};
const rgbToHex = (r, g, b) => {
  return decToHex(r) + decToHex(g) + decToHex(b);
};
const rgbToDec = (r, g, b) => {
  return parseInt(rgbToHex(r, g, b), 16);
};
module.exports = { decToHex, rgbToHex, rgbToDec };
