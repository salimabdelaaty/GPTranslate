document.addEventListener('DOMContentLoaded', function () {

  // Smooth scrolling for anchor links
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
      e.preventDefault();
      const target = document.querySelector(this.getAttribute('href'));
      if (target) {
        target.scrollIntoView({
          behavior: 'smooth',
          block: 'start'
        });
      }
    });
  });

  // Optional fade-in animation
  const observerOptions = {
    threshold: 0.1,
    rootMargin: '0px 0px -50px 0px'
  };
  const observer = new IntersectionObserver(function (entries) {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        entry.target.style.opacity = '1';
        entry.target.style.transform = 'translateY(0)';
      }
    });
  }, observerOptions);

  document
    .querySelectorAll('.feature-card, .screenshot-item, .tech-item, .example-card')
    .forEach(el => {
      el.style.opacity = '0';
      el.style.transform = 'translateY(20px)';
      el.style.transition = 'opacity 0.6s ease, transform 0.6s ease';
      observer.observe(el);
    });

  // Copy code functionality for code blocks
  document.querySelectorAll('pre code').forEach(block => {
    const button = document.createElement('button');
    button.className = 'copy-button';
    button.textContent = 'Copy';
    button.onclick = function () {
      navigator.clipboard.writeText(block.textContent).then(() => {
        button.textContent = 'Copied!';
        setTimeout(() => {
          button.textContent = 'Copy';
        }, 2000);
      });
    };

    const wrapper = document.createElement('div');
    wrapper.className = 'code-wrapper';
    block.parentNode.insertBefore(wrapper, block);
    wrapper.appendChild(block);
    wrapper.appendChild(button);
  });

  // External link indicator
  document.querySelectorAll('a[href^="http"]').forEach(link => {
    if (!link.hostname === window.location.hostname) {
      link.setAttribute('target', '_blank');
      link.setAttribute('rel', 'noopener noreferrer');
      const icon = document.createElement('i');
      icon.className = 'bi bi-arrow-up-right-square';
      icon.style.marginLeft = '4px';
      icon.style.fontSize = '0.8em';
      link.appendChild(icon);
    }
  });

  // Table of contents in content-body
  const contentBody = document.querySelector('.content-body');
  if (contentBody) {
    const headings = contentBody.querySelectorAll('h2, h3');
    if (headings.length > 2) {
      const toc = document.createElement('div');
      toc.className = 'table-of-contents';
      toc.innerHTML = '<h3>Table of Contents</h3><ul></ul>';

      const tocList = toc.querySelector('ul');
      headings.forEach((heading, index) => {
        const id = `heading-${index}`;
        heading.id = id;
        const li = document.createElement('li');
        li.className = heading.tagName.toLowerCase();
        const a = document.createElement('a');
        a.href = `#${id}`;
        a.textContent = heading.textContent;
        li.appendChild(a);
        tocList.appendChild(li);
      });
      contentBody.insertBefore(toc, contentBody.firstChild);
    }
  }

  // Theme detection with Pico
  function updateTheme() {
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
  }
  updateTheme();
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', updateTheme);

  // Lazy load images
  const lazyImages = document.querySelectorAll('img[loading="lazy"]');
  if ('IntersectionObserver' in window) {
    const imageObserver = new IntersectionObserver((entries, observer) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          const img = entry.target;
          if (img.dataset.src) {
            img.src = img.dataset.src;
          }
          img.classList.remove('lazy');
          imageObserver.unobserve(img);
        }
      });
    });
    lazyImages.forEach(img => imageObserver.observe(img));
  }
});