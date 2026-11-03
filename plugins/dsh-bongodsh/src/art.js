// Display the stored PNG layers without changing their pixels or silhouettes.
export function createArtwork(document, characters) {
  const namespace = 'http://www.w3.org/2000/svg';
  const svg = document.createElementNS(namespace, 'svg');
  svg.setAttribute('viewBox', '0 0 300 252');
  svg.setAttribute('aria-hidden', 'true');

  function layer(attribute, x, y, width, height) {
    const group = document.createElementNS(namespace, 'g');
    group.setAttribute(attribute, '');
    const image = document.createElementNS(namespace, 'image');
    for (const [name, value] of Object.entries({ x, y, width, height, preserveAspectRatio: 'xMidYMax meet' })) {
      image.setAttribute(name, String(value));
    }
    group.append(image);
    svg.append(group);
    return { group, image };
  }

  const body = layer('data-body', 6, 6, 288, 230);
  // Overlay the hands on the lower front of the body instead of reserving a row below it.
  const hands = [layer('data-left-hand', 51, 187, 50, 42), layer('data-right-hand', 127, 187, 50, 42)];
  const bodyBottom = 236;

  function render(positions) {
    const activity = Math.max(0, Math.min(1, Math.max(...positions)));
    body.group.setAttribute('transform', `translate(150 ${bodyBottom}) scale(${1 + activity * 0.02} ${1 - activity * 0.035}) translate(-150 ${-bodyBottom})`);
    hands.forEach(({ group }, index) => {
      const press = Math.max(-0.2, Math.min(1.15, positions[index]));
      const x = index === 0 ? 76 : 152;
      group.setAttribute('transform', `translate(${x} ${208 + press * 3}) scale(${1 + press * 0.08} ${1 - press * 0.18}) translate(${-x} -208)`);
    });
  }

  function select(id) {
    const character = characters.find(item => item.id === id) ?? characters[0];
    body.image.setAttribute('href', character.body);
    hands.forEach(({ group, image }, index) => {
      const source = character[index === 0 ? 'leftHand' : 'rightHand'];
      group.setAttribute('display', source ? 'inline' : 'none');
      if (source) image.setAttribute('href', source);
      else image.removeAttribute('href');
    });
    svg.dataset.character = character.id;
    render([0, 0]);
    return character;
  }

  return { svg, render, select };
}
