/* Shared runtime for BioOKF Studio HyperFrames mockups.
   - fitStage(): scales the fixed 1280x720 .stage1280 to "contain" inside the iframe
   - HF.register(id, tl): registers a paused GSAP timeline on window.__timelines[id]
     (so the official <hyperframes-player> / CLI can seek+render it deterministically),
     and - when loaded standalone with ?selfplay or as the top frame - plays it on loop. */
(function () {
  'use strict';
  var BASE_W = 1280, BASE_H = 720;

  function fitStage() {
    var stage = document.querySelector('.stage1280');
    if (!stage) return;
    var vw = window.innerWidth, vh = window.innerHeight;
    var s = Math.min(vw / BASE_W, vh / BASE_H);
    var x = (vw - BASE_W * s) / 2, y = (vh - BASE_H * s) / 2;
    stage.style.transform = 'translate(' + x + 'px,' + y + 'px) scale(' + s + ')';
  }
  window.addEventListener('resize', fitStage);
  if (document.readyState !== 'loading') fitStage();
  else document.addEventListener('DOMContentLoaded', fitStage);

  var params = new URLSearchParams(window.location.search);
  var selfplay = params.has('selfplay') || window.self === window.top;

  window.HF = {
    fit: fitStage,
    selfplay: selfplay,
    register: function (id, tl) {
      window.__timelines = window.__timelines || {};
      window.__timelines[id] = tl;          // HyperFrames player / CLI drives this
      if (selfplay && tl && typeof tl.play === 'function') {
        // loop with a short hold so the story reads clearly each pass
        if (typeof tl.repeat === 'function') { tl.repeat(-1); tl.repeatDelay(0.9); }
        tl.play();
      }
      return tl;
    }
  };
})();
