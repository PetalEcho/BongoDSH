// Display the stored PNG layers without changing their pixels or silhouettes.
export function createArtwork(document, characters) {
  const namespace = 'http://www.w3.org/2000/svg';
  const svg = document.createElementNS(namespace, 'svg');
  svg.setAttribute('viewBox', '0 0 300 252');
  svg.setAttribute('aria-hidden', 'true');

  function layer(attribute) {
    const group = document.createElementNS(namespace, 'g');
    group.setAttribute(attribute, '');
    const image = document.createElementNS(namespace, 'image');
    group.append(image);
    svg.append(group);
    return { group, image };
  }

  const body = layer('data-body');
  const hands = [layer('data-left-hand'), layer('data-right-hand')];
  const bodyBottom = 248;
  const defaultHandLayout = [{ centerX: 56, width: 36, height: 28 }, { centerX: 188, width: 36, height: 28 }];
  let handLayout = defaultHandLayout;

  function fitImage(image, bounds, x, y, width, height) {
    let geometry = { x, y, width, height, preserveAspectRatio: 'xMidYMax meet' };
    if (Array.isArray(bounds) && bounds.length === 6 && bounds.every(Number.isFinite)
      && bounds.slice(2).every(value => value > 0)) {
      const [left, top, visibleWidth, visibleHeight, canvasWidth, canvasHeight] = bounds;
      const scale = Math.min(width / visibleWidth, height / visibleHeight);
      // Fit the visible silhouette, keeping the original PNG and its alpha channel intact.
      geometry = {
        x: x + (width - visibleWidth * scale) / 2 - left * scale,
        y: y + height - (visibleHeight + top) * scale,
        width: canvasWidth * scale, height: canvasHeight * scale, preserveAspectRatio: 'none',
      };
    }
    for (const [name, value] of Object.entries(geometry)) image.setAttribute(name, String(value));
  }

  function render(positions) {
    const activity = Math.max(0, Math.min(1, Math.max(...positions)));
    body.group.setAttribute('transform', `translate(150 ${bodyBottom}) scale(${1 + activity * 0.02} ${1 - activity * 0.035}) translate(-150 ${-bodyBottom})`);
    hands.forEach(({ group }, index) => {
      const press = Math.max(-0.2, Math.min(1.15, positions[index]));
      const x = handLayout[index].centerX;
      group.setAttribute('transform', `translate(${x} ${bodyBottom + press * 0.7}) scale(${1 + press * 0.06} ${1 - press * 0.15}) translate(${-x} ${-bodyBottom})`);
    });
  }

  function select(id) {
    const character = characters.find(item => item.id === id) ?? characters[0];
    body.image.setAttribute('href', character.body);
    fitImage(body.image, character.bounds?.body, 6, 6, 288, bodyBottom - 6);
    handLayout = character.handLayout ?? defaultHandLayout;
    hands.forEach(({ group, image }, index) => {
      const name = index === 0 ? 'leftHand' : 'rightHand';
      const source = character[name];
      group.setAttribute('display', source ? 'inline' : 'none');
      if (source) {
        image.setAttribute('href', source);
        const { centerX, width, height } = handLayout[index];
        fitImage(image, character.bounds?.[name], centerX - width / 2, bodyBottom - height, width, height);
      } else image.removeAttribute('href');
    });
    svg.dataset.character = character.id;
    render([0, 0]);
    return character;
  }

  return { svg, render, select };
}
