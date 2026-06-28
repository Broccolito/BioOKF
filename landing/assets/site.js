/* BioOKF landing - shared behaviour
   - scroll-reveal fades
   - mobile nav toggle
   - HyperFrames embeds: official <hyperframes-player> primary, self-looping
     <iframe> fallback if the CDN web component never registers
   - docs scrollspy
*/
(function () {
  'use strict';

  /* ---- scroll reveal ---- */
  var faders = document.querySelectorAll('.fade');
  if ('IntersectionObserver' in window && faders.length) {
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (e.isIntersecting) { e.target.classList.add('in'); io.unobserve(e.target); }
      });
    }, { rootMargin: '0px 0px -8% 0px', threshold: 0.08 });
    faders.forEach(function (el, i) { el.style.transitionDelay = (Math.min(i, 6) * 45) + 'ms'; io.observe(el); });
  } else {
    faders.forEach(function (el) { el.classList.add('in'); });
  }

  /* ---- mobile nav ---- */
  var burger = document.querySelector('.nav-burger');
  var links = document.querySelector('.nav-links');
  if (burger && links) {
    burger.addEventListener('click', function () { links.classList.toggle('open'); });
    links.addEventListener('click', function (e) { if (e.target.tagName === 'A') links.classList.remove('open'); });
  }

  /* ---- HyperFrames embeds ---- */
  var embeds = Array.prototype.slice.call(document.querySelectorAll('.hf-embed'));
  if (embeds.length) {
    var PLAYER_SRC = 'https://cdn.jsdelivr.net/npm/@hyperframes/player';

    function mountPlayer(el) {
      var p = document.createElement('hyperframes-player');
      p.setAttribute('src', el.dataset.src);
      p.setAttribute('autoplay', '');
      p.setAttribute('loop', '');
      p.setAttribute('muted', '');
      p.setAttribute('controls', '');
      el.innerHTML = '';
      el.appendChild(p);
      el.dataset.mode = 'player';
    }
    function mountIframe(el) {
      var f = document.createElement('iframe');
      // self-looping fallback: the composition plays its own GSAP timeline on ?selfplay
      f.src = el.dataset.src + (el.dataset.src.indexOf('?') < 0 ? '?' : '&') + 'selfplay=1';
      f.setAttribute('loading', 'lazy');
      f.setAttribute('title', el.dataset.title || 'BioOKF Studio mockup');
      f.setAttribute('scrolling', 'no');
      el.innerHTML = '';
      el.appendChild(f);
      el.dataset.mode = 'iframe';
    }
    function mountAll(usePlayer) {
      embeds.forEach(function (el) {
        if (el.dataset.mode) return;
        if (usePlayer && window.customElements && customElements.get('hyperframes-player')) mountPlayer(el);
        else mountIframe(el);
      });
    }

    // Try the official player; fall back to plain iframes if it never registers.
    var settled = false;
    function settle(usePlayer) { if (settled) return; settled = true; mountAll(usePlayer); }

    try {
      var s = document.createElement('script');
      s.type = 'module';
      s.src = PLAYER_SRC;
      s.onload = function () {
        if (window.customElements && customElements.whenDefined) {
          customElements.whenDefined('hyperframes-player').then(function () { settle(true); });
        }
      };
      s.onerror = function () { settle(false); };
      document.head.appendChild(s);
    } catch (err) { settle(false); }

    // Hard timeout: whatever happened, show the mockups within 2.2s.
    setTimeout(function () { settle(!!(window.customElements && customElements.get('hyperframes-player'))); }, 2200);
  }

  /* ---- docs scrollspy ---- */
  var spyLinks = document.querySelectorAll('.docs-side a[href^="#"]');
  if (spyLinks.length && 'IntersectionObserver' in window) {
    var map = {};
    spyLinks.forEach(function (a) { map[a.getAttribute('href').slice(1)] = a; });
    var current = null;
    var spy = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (e.isIntersecting) {
          if (current) current.classList.remove('active');
          current = map[e.target.id];
          if (current) current.classList.add('active');
        }
      });
    }, { rootMargin: '-76px 0px -70% 0px', threshold: 0 });
    document.querySelectorAll('.doc-sec[id]').forEach(function (sec) { spy.observe(sec); });
  }
})();
