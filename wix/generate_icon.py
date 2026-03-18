"""mcuforge 아이콘 생성 스크립트 - 추상 큐브 + 회로 흐름 디자인"""
from PIL import Image, ImageDraw
import math

def draw_icon(size):
    """주어진 크기로 mcuforge 아이콘 렌더링"""
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    s = size / 512  # 스케일 팩터

    # 배경: 둥근 모서리 사각형
    bg_margin = int(16 * s)
    bg_radius = int(64 * s)
    draw.rounded_rectangle(
        [bg_margin, bg_margin, size - bg_margin, size - bg_margin],
        radius=bg_radius,
        fill=(22, 33, 62)  # #16213e
    )

    # 아이소메트릭 큐브 좌표 (중심: 240, 230)
    cx, cy = 240 * s, 230 * s
    # 큐브 꼭짓점 (아이소메트릭)
    top     = (240 * s, 110 * s)
    mid     = (240 * s, 230 * s)
    bot     = (240 * s, 350 * s)
    left_t  = (148 * s, 175 * s)
    left_b  = (148 * s, 295 * s)
    right_t = (332 * s, 175 * s)
    right_b = (332 * s, 295 * s)

    # 큐브 좌측면 (#6c5ce7 → #4834d4)
    draw.polygon([mid, bot, left_b, left_t], fill=(88, 68, 210))

    # 큐브 우측면 (#0984e3 → #0652DD)
    draw.polygon([mid, bot, right_b, right_t], fill=(9, 110, 210))

    # 큐브 상단면 (#a29bfe → #6c5ce7)
    draw.polygon([top, right_t, mid, left_t], fill=(140, 122, 254))

    # 큐브 엣지 하이라이트
    if size >= 48:
        edge_w = max(1, int(1.5 * s))
        draw.line([top, mid], fill=(162, 155, 254, 150), width=edge_w)

    # 회로 트레이스 (시안 #00cec9)
    trace_color = (0, 206, 201)
    trace_color_dim = (116, 185, 255)
    node_r = max(2, int(5 * s))
    trace_w = max(1, int(3 * s))
    trace_w2 = max(1, int(2.5 * s))

    # 트레이스 1: 우측면 → 우하단
    pts1 = [(332*s, 230*s), (365*s, 250*s), (365*s, 310*s), (395*s, 330*s)]
    draw.line(pts1, fill=trace_color, width=trace_w, joint='curve')
    draw.ellipse([395*s - node_r, 330*s - node_r, 395*s + node_r, 330*s + node_r],
                 fill=trace_color)

    # 트레이스 2: 우측면 → 오른쪽
    pts2 = [(332*s, 200*s), (370*s, 220*s), (420*s, 220*s)]
    draw.line(pts2, fill=trace_color, width=trace_w2, joint='curve')
    node_r2 = max(2, int(4 * s))
    draw.ellipse([420*s - node_r2, 220*s - node_r2, 420*s + node_r2, 220*s + node_r2],
                 fill=trace_color)

    # 트레이스 3: 하단 → 아래
    pts3 = [(240*s, 350*s), (240*s, 390*s), (280*s, 410*s), (280*s, 440*s)]
    draw.line(pts3, fill=trace_color, width=trace_w2, joint='curve')
    draw.ellipse([280*s - node_r2, 440*s - node_r2, 280*s + node_r2, 440*s + node_r2],
                 fill=trace_color)

    # 트레이스 4: 좌측 → 좌하단 (연한)
    if size >= 32:
        tw3 = max(1, int(2 * s))
        pts4 = [(148*s, 240*s), (115*s, 260*s), (115*s, 330*s)]
        draw.line(pts4, fill=(*trace_color_dim, 130), width=tw3, joint='curve')
        node_r3 = max(1, int(3.5 * s))
        draw.ellipse([115*s - node_r3, 330*s - node_r3, 115*s + node_r3, 330*s + node_r3],
                     fill=(*trace_color_dim, 130))

    # 트레이스 5: 상단 → 우상단 (연한)
    if size >= 48:
        pts5 = [(290*s, 140*s), (340*s, 115*s), (390*s, 115*s)]
        draw.line(pts5, fill=(*trace_color_dim, 100), width=tw3, joint='curve')
        draw.ellipse([390*s - node_r3, 115*s - node_r3, 390*s + node_r3, 115*s + node_r3],
                     fill=(*trace_color_dim, 100))

    # 큐브 상단면 회로 노드
    if size >= 32:
        nr = max(2, int(4 * s))
        draw.ellipse([220*s - nr, 170*s - nr, 220*s + nr, 170*s + nr],
                     fill=trace_color)
        nr2 = max(1, int(3 * s))
        draw.ellipse([260*s - nr2, 185*s - nr2, 260*s + nr2, 185*s + nr2],
                     fill=trace_color)
        if size >= 48:
            nw = max(1, int(1.5 * s))
            draw.line([(220*s, 170*s), (260*s, 185*s)], fill=(*trace_color, 160), width=nw)

    return img


def main():
    sizes = [256, 48, 32, 16]
    images = [draw_icon(s) for s in sizes]

    # ICO 저장
    images[0].save(
        'wix/mcuforge.ico',
        format='ICO',
        sizes=[(s, s) for s in sizes],
        append_images=images[1:]
    )
    print(f"mcuforge.ico 생성 완료 (해상도: {sizes})")

    # 미리보기용 PNG (256px)
    images[0].save('wix/mcuforge-256.png')
    print("mcuforge-256.png 미리보기 저장 완료")


if __name__ == '__main__':
    main()
