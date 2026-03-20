use std::hash::Hash;
use std::fmt::Debug;

use crate::fa::FA;
use crate::wlrendering::render;


// Terrible way to do this but it works. 
#[allow(dead_code)]
#[derive(Debug, Hash)]
pub struct FFA<'a, N: Debug> {pub q: &'a Vec<N>,pub sigma: &'a usize,pub t: &'a Vec<(N, usize, N)>,pub q0: &'a N,pub f: &'a Vec<N>,}
impl<N: Clone + Debug + Eq + Hash> Debug for FA<N> {fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {(FFA {q: &self.q, sigma: &self.sigma, t: &self.t, q0: &self.q0, f: &self.f}).fmt(f)}}
impl<N: Clone + Debug + Eq + Hash> Hash for FA<N> {fn hash<H: std::hash::Hasher>(&self, state: &mut H) {(FFA {q: &self.q, sigma: &self.sigma, t: &self.t, q0: &self.q0, f: &self.f}).hash(state);}}

impl<N: Clone + Debug + Hash + Eq> FA<N> {
    #[allow(dead_code)]
    pub fn fmt_named_wl(&self, name: &str) -> String {
        fn fmt_v<T: Debug>(v: &T) -> String {
            format!("{:?}", v).replace("[", "{").replace("]", "}")
        }

        let fmt_nodes = fmt_v(&self.q);
        let fmt_edges = "{".to_string() + &self.t.iter().map(|(n1, x, n2)| format!("Labeled[{} \\[DirectedEdge] {}, {x}]", fmt_v(n1), fmt_v(n2))).collect::<Vec<String>>().join(", ") + "}";
        let fmt_styles = fmt_v(&self.q0);
        let fmt_halts = self.f.iter().map(|n| format!("{:?} -> Blue", n)).collect::<Vec<String>>().join(", ").replace("[", "{").replace("]", "}");
        let o = format!("EdgeTaggedGraph[{fmt_nodes}, {fmt_edges}, VertexLabels -> \"Name\", VertexStyle -> ({fmt_halts}, {fmt_styles} -> Red), ImageSize -> Large]").replace("(", "{").replace(")", "}").replace("\'", "\"");
        
        format!("Labeled[{o}, {name:?}]")
    }
    #[allow(dead_code)]
    pub fn fmt_wl(&self) -> String {
        self.fmt_named_wl("")
    }
    #[allow(dead_code)]
    pub fn print_named_wl(&self, name: &str) {
        println!("{}", self.fmt_named_wl(name));
    }
    #[allow(dead_code)]
    pub fn print_wl(&self) {
        println!("{}", self.fmt_wl());
    }
    #[allow(dead_code)]
    pub fn debug_named_wl(&self, name: &str) {
        self.print_named_wl(name);
        eprintln!("{name}: {self:?}");
    }
    #[allow(dead_code)]
    pub fn render_named_wl(&self, name: &str) {
        let path = std::env::current_dir().expect("couldnt get current dir").display().to_string() + "/renders/" + name;
        render(self.fmt_named_wl(name), &path);
    }
    #[allow(dead_code)]
    pub fn render_timestamped_wl(&self, name: &str) {
        let time = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs((std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() / 86400) * 86400)).unwrap().as_micros();
        self.render_named_wl(&format!("{time}_{name}"));
    }
}