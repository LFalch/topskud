use ggez::{Context, GameResult};
use ggez::graphics::{self, DrawMode, Rect, Point2, Vector2};
use ggez::nalgebra::distance;
use std::mem;

pub trait Position: Clone {
    #[inline]
    fn x(&self) -> f32 {
        self.pos().x
    }
    #[inline]
    fn y(&self) -> f32 {
        self.pos().y
    }
    fn pos(&self) -> &Point2;
    fn pos_mut(&mut self) -> &mut Point2;
}

impl Position for Point2 {
    #[inline]
    fn x(&self) -> f32 {
        self.x
    }
    #[inline]
    fn y(&self) -> f32 {
        self.y
    }
    #[inline]
    fn pos(&self) -> &Self {
        self
    }
    #[inline]
    fn pos_mut(&mut self) -> &mut Self {
        self
    }
}

pub enum QTree<T: Position> {
    Divided{
        ul: Box<QTree<T>>,
        ur: Box<QTree<T>>,
        ll: Box<QTree<T>>,
        lr: Box<QTree<T>>,
    },
    Points(Vec<T>),
}

impl<T: Position> QTree<T> {
    fn new_divided() -> Self {
        QTree::Divided {
            ul: Box::new(QTree::Points(Vec::new())),
            ur: Box::new(QTree::Points(Vec::new())),
            ll: Box::new(QTree::Points(Vec::new())),
            lr: Box::new(QTree::Points(Vec::new())),
        }
    }
    fn insert(&mut self, elem: T, c: Point2, w: f32, h: f32, cap: usize) {
        let p = *elem.pos();
        let in_bounds = p.x >= c.x && p.y >= c.y && p.x < c.x + w && p.y < c.y + h;
        if !in_bounds {
            return;
        }

        match *self {
            QTree::Points(ref mut ps) => {
                if ps.len() < cap {
                    return ps.push(elem);
                }
            }
            QTree::Divided{ref mut ul,ref mut ur,ref mut ll,ref mut lr} => {
                let w = w / 2.;
                let h = h / 2.;
                ul.insert(elem.clone(), c, w, h, cap);
                ur.insert(elem.clone(), c + Vector2::new(w, 0.), w, h, cap);
                ll.insert(elem.clone(), c + Vector2::new(0., h), w, h, cap);
                lr.insert(elem, c + Vector2::new(w, h), w, h, cap);
                return
            }
        }
        if let QTree::Points(ps) = mem::replace(self, QTree::new_divided()) {
            for p in ps {
                self.insert(p, c, w, h, cap);
            }
            self.insert(elem, c, w, h, cap);
        } else {
            unreachable!()
        }
    }
    fn draw(&self, ctx: &mut Context, c: Point2, w: f32, h: f32) -> GameResult<()> {
        graphics::rectangle(ctx, DrawMode::Line(1.), Rect::new(c.x, c.y, w, h))?;
        match *self {
            QTree::Points(ref ps) => graphics::points(ctx, &*ps.iter().map(|p| *p.pos()).collect::<Vec<_>>(), 2.),
            QTree::Divided{ref ul, ref ur, ref ll, ref lr} => {
                let w = w / 2.;
                let h = h / 2.;
                ul.draw(ctx, c, w, h)?;
                ur.draw(ctx, c + Vector2::new(w, 0.), w, h)?;
                ll.draw(ctx, c + Vector2::new(0., h), w, h)?;
                lr.draw(ctx, c + Vector2::new(w, h), w, h)
            }
        }
    }
    fn query_points<F: FnMut(Point2, f32, f32) -> bool>(&self, c: Point2, w: f32, h: f32, quad_condition: &mut F) -> Vec<&T> {
        let mut ret_ps = Vec::new();
        if quad_condition(c, w, h) {
            match *self {
                QTree::Points(ref ps) => ret_ps.extend(ps),
                QTree::Divided{ref ul, ref ur, ref ll, ref lr} => {
                    let w = w / 2.;
                    let h = h / 2.;
                    ret_ps.extend(ul.query_points(c, w, h, quad_condition));
                    ret_ps.extend(ur.query_points(c + Vector2::new(w, 0.), w, h, quad_condition));
                    ret_ps.extend(ll.query_points(c + Vector2::new(0., h), w, h, quad_condition));
                    ret_ps.extend(lr.query_points(c + Vector2::new(w, h), w, h, quad_condition));
                }
            }
        }
        ret_ps
    }
}

pub struct QuadTree<T: Position> {
    tree: QTree<T>,
    capacity: usize,
    width: f32,
    height: f32,
}

impl<T: Position> QuadTree<T> {
    pub fn new(width: f32, height: f32, capacity: usize) -> Self {
        QuadTree {
            tree: QTree::Points(Vec::with_capacity(capacity)),
            capacity,
            width,
            height,
        }
    }
    pub fn insert(&mut self, elem: T) {
        self.tree.insert(elem, Point2::origin(), self.width, self.height, self.capacity);
    }
    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        self.tree.draw(ctx, Point2::origin(), self.width, self.height)
    }
    pub fn query_circular(&self, point: Point2, radius: f32) -> Vec<&T> {
        let mut ret = self.tree.query_points(Point2::origin(), self.width, self.height, &mut |c, w, h| {
            let w = w / 2.;
            let h = h / 2.;
            let dist = point - c - Vector2::new(w, h);
            let dist_x = dist.x.abs();
            let dist_y = dist.y.abs();

            if dist_x > w + radius || dist_y > h + radius {
                return false
            }
            if dist_x <= w || dist_y <= h {
                return true
            }
            let corner_distance_sq = (dist_x - w).hypot(dist_y - h);
            corner_distance_sq <= radius * radius
        });
        ret.retain(|p| distance(p.pos(), &point) <= radius);
        ret
    }
    pub fn query_rectangular(&self, lr_corner: Point2, width: f32, height: f32) -> Vec<&T> {
        let mut ret = self.tree.query_points(Point2::origin(), self.width, self.height, &mut |c, w, h| {
            c.x <= lr_corner.x + width && c.x + w >= lr_corner.x &&
            c.y <= lr_corner.y + height && c.y + h >= lr_corner.y
        });
        ret.retain(|p| lr_corner.x <= p.x() && lr_corner.x + width >= p.x() && lr_corner.y <= p.y() && lr_corner.y + height >= p.y());
        ret
    }
}
