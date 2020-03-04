/*
 *   █████╗ ███╗   ██╗██╗███╗   ███╗ █████╗ ███████╗ ██████╗██╗██╗        ██╗███████╗
 *  ██╔══██╗████╗  ██║██║████╗ ████║██╔══██╗██╔════╝██╔════╝██║██║        ██║██╔════╝
 *  ███████║██╔██╗ ██║██║██╔████╔██║███████║███████╗██║     ██║██║        ██║███████╗
 *  ██╔══██║██║╚██╗██║██║██║╚██╔╝██║██╔══██║╚════██║██║     ██║██║   ██   ██║╚════██║
 *  ██║  ██║██║ ╚████║██║██║ ╚═╝ ██║██║  ██║███████║╚██████╗██║██║██╗╚█████╔╝███████║
 *  ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝╚═╝     ╚═╝╚═╝  ╚═╝╚══════╝ ╚═════╝╚═╝╚═╝╚═╝ ╚════╝ ╚══════╝
 * COPYRIGHT 2018 THEGREATRAMBLER github.com/TheGreatRambler/AnimASCII
 * MIT License
 */
(function(root, factory) {
    if (typeof define === 'function' && define.amd) {
        define(["ROT"], factory);
    } else if (typeof module === 'object' && module.exports) {
        module.exports = factory(require("ROT"));
    } else {
        root["animascii"] = factory(root["ROT"]);
    }
}(typeof self !== 'undefined' ? self : this, function(ROT) {
    var animascii = function(inputoptions, callback) {
        var sourcearraybool = Array.isArray(inputoptions.src);
        var options = {};
        this.iteration = 0;
        var that = this;
        this.stopbool = false;
        
        function setDefaults() {
            if (typeof inputoptions.repeat === "undefined") {
                options.repeat = 1;
            } else {
                options.repeat = inputoptions.repeat;
            }
            if (typeof inputoptions.letter_padding === "undefined") {
                options.letter_padding = 1;
            } else {
                options.letter_padding = inputoptions.letter_padding;
            }
            if (typeof inputoptions.font_family === "undefined") {
                options.font_family = "monospace";
            } else {
                options.font_family = inputoptions.font_family;
            }
            if (typeof inputoptions.font_size === "undefined") {
                options.font_size = 25;
            } else {
                options.font_size = inputoptions.font_size;
            }
            if (typeof inputoptions.background_color === "undefined") {
                options.background_color = "white";
            } else {
                options.background_color = inputoptions.background_color;
            }
            if (typeof inputoptions.foreground_color === "undefined") {
                options.foreground_color = "black";
            } else {
                options.foreground_color = inputoptions.foreground_color;
            }
            if (typeof inputoptions.delay === "undefined") {
                options.delay = 200;
            } else {
                options.delay = inputoptions.delay;
            }
        }

        function parsetextdoc(options, method, func) {
            function find(collection, predicate, fromIndex) {
                var result;
                collection.forEach(function(value) {
                    if (value[predicate[0]] === value[predicate[1]]) {
                        result = value;
                    }
                });
                return result;
            }
            var xmlhttp;
            if (window.XMLHttpRequest) {
                xmlhttp = new XMLHttpRequest();
            } else {
                xmlhttp = new ActiveXObject("Microsoft.XMLHTTP");
            }
            xmlhttp.onreadystatechange = function() {
                if (this.readyState == 4 && this.status == 200) {
                    var lines = this.responseText.split('\n');
                    var data = {};
                    lines.forEach(function(result) {
                        var linedata = method.find(function(value) {
                            return value.symbol === result.charAt(0);
                        });
                        if (typeof linedata !== "undefined") {
                            if (Array.isArray(data[linedata.name]) === false) {
                                data[linedata.name] = [];
                            }
                            data[linedata.name].push(result.substr(1));
                        }
                    });
                    func(data);
                }
            };
            xmlhttp.open("GET", options.src, true);
            xmlhttp.send();
        }

        setDefaults();

        that.asciiscreen = new ROT.Display({
            fontSize: options.font_size,
            bg: options.background_color,
            fg: options.foreground_color,
            fontFamily: options.font_family,
            spacing: options.letter_padding
        });
        ROT.Display.Rect.cache = true;
        inputoptions.display.appendChild(this.asciiscreen.getContainer());
        
        this.stop = function() {
            that.stopbool = true;
        }

        function draw(n, data, numofframes) {
            var width = data.widthheight[0];
            var height = data.widthheight[1];
            if (n < numofframes) {
                var startval = n * height;
                for (let t = 0; t < width; t++) {
                    for (let g = 0; g < height; g++) {
                        that.asciiscreen.draw(t, g, data.frames[startval + g][t]);
                    }
                }
                if (that.stopbool) {
                    if (callback) {
                        callback();
                    }
                } else {
                    setTimeout(function() {
                        draw(++n, data, numofframes);
                    }, data.frametime[n]);
                }
            } else {
                if (options.repeat === -1 || that.iteration < options.repeat) {
                    that.iteration++;
                    draw(0, data, numofframes);
                } else {
                    if (callback) {
                        callback();
                    }
                }
            }
        }

        if (sourcearraybool === false) {
            parsetextdoc({
                src: inputoptions.src
            }, [{
                symbol: "#",
                name: "widthheight"
            }, {
                symbol: "^",
                name: "frametime"
            }, {
                symbol: "%",
                name: "frames"
            }, {
                symbol: "_",
                name: "comments"
            }, {
                symbol: "+",
                name: "name"
            }], function(filedata) {
                var width = filedata.widthheight[0];
                var height = filedata.widthheight[1];
                that.asciiscreen.setOptions({
                    width: width,
                    height: height
                });
                var cornerx = 0;
                var cornery = 0;
                var numofframes = filedata.frames.length / height;
                draw(0, filedata, numofframes);
            });
        } else {
            this.asciiscreen.setOptions({
                width: inputoptions.src[0][0].length,
                height: inputoptions.src[0].length
            });
            draw(0, {
                widthheight: [inputoptions.src[0][0].length, inputoptions.src[0].length],
                frametime: Array(inputoptions.src.length).fill(options.delay),
                frames: [].concat.apply([], inputoptions.src)
            }, inputoptions.src.length);
        }
    };
    return animascii;
}));