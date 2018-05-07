(function () {
  function outer(modules, cache, entry) {
    var previousRequire = typeof require == 'function' && require;

    function newRequire(name, jumped){
      if(!cache[name]) {
        if(!modules[name]) {
          var currentRequire = typeof require == 'function' && require;
          if (!jumped && currentRequire) return currentRequire(name, true);

          if (previousRequire) return previousRequire(name, true);
          var err = new Error('Cannot find module \\'' + name + '\\'');
          err.code = 'MODULE_NOT_FOUND';
          throw err;
        }
        var m = cache[name] = {exports:{}};
        modules[name][0].call(m.exports, function(x){
          var id = modules[name][1][x];
          return newRequire(id ? id : x);
        },m,m.exports,outer,modules,cache,entry);
      }
      return cache[name].exports;
    }
    for(var i=0;i<entry.length;i++) newRequire(entry[i]);

    return newRequire;
  }

  return outer;
})()
