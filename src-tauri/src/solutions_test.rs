//! Pruebas de soluciones canónicas para los 9 niveles.
//!
//! Cada test verifica que los comandos iptables del jugador producen el
//! veredicto correcto para cada prueba del nivel correspondiente.
//!
//! Equivalente al `TestLevelSolutions` del motor Go de referencia.

#[cfg(test)]
mod tests {
    use crate::engine::packet::{
        parse_state_bit, Packet, Proto, STATE_ESTABLISHED, STATE_NEW,
    };
    use crate::engine::parser::parse_line;
    use crate::engine::pipeline::{evaluate, FinalVerdict};
    use crate::engine::ruleset::Ruleset;
    use crate::engine::topology::{topology_from_level, Topology};
    use crate::levels::{Level, load_all};

    // ──────────────────────────────────────────────────────────────────
    // Helpers
    // ──────────────────────────────────────────────────────────────────

    /// Construye una Topology a partir de la descripción de red del nivel.
    fn topo_from_level(lvl: &Level) -> Topology {
        topology_from_level(
            &lvl.red.firewall_ip,
            &lvl.red.interfaces,
            &lvl.red.hosts,
        )
    }

    /// Aplica políticas + reglas iniciales del nivel + comandos del jugador.
    /// Devuelve el ruleset listo para evaluar paquetes.
    fn setup_level(lvl: &Level, solution: &[&str]) -> Ruleset {
        let mut rs = Ruleset::new();

        // 1. Políticas del nivel
        for (chain, policy_str) in &lvl.politicas {
            let cmd_line = format!("-P {chain} {policy_str}");
            let cmd = parse_line(&cmd_line)
                .unwrap_or_else(|_| panic!("Error parseando política: {cmd_line}"));
            rs.apply(&cmd)
                .unwrap_or_else(|e| panic!("Error aplicando política {cmd_line}: {e}"));
        }

        // 2. Reglas iniciales del nivel
        for line in &lvl.reglas_iniciales {
            if line.trim().is_empty() {
                continue;
            }
            let cmd = parse_line(line)
                .unwrap_or_else(|_| panic!("Error parseando regla inicial: {line}"));
            rs.apply(&cmd)
                .unwrap_or_else(|e| panic!("Error aplicando regla inicial {line}: {e}"));
        }

        // 3. Comandos del jugador (solución canónica)
        for line in solution {
            if line.trim().is_empty() {
                continue;
            }
            let cmd = parse_line(line)
                .unwrap_or_else(|_| panic!("Error parseando comando solución: {line}"));
            rs.apply(&cmd)
                .unwrap_or_else(|e| panic!("Error aplicando comando solución {line}: {e}"));
        }

        rs
    }

    /// Convierte string "NEW" | "ESTABLISHED" | "RELATED" → bitmask.
    fn parse_state(s: &str) -> u8 {
        s.split(',')
            .map(|tok| parse_state_bit(tok.trim()))
            .fold(0u8, |acc, b| acc | b)
    }

    /// Ejecuta todas las pruebas del nivel contra el ruleset + topología dados.
    fn run_pruebas(lvl: &Level, rs: &Ruleset, topo: &Topology) {
        for prueba in &lvl.pruebas {
            let pkt = Packet {
                src_ip: prueba.src_ip.clone(),
                dst_ip: prueba.dst_ip.clone(),
                dst_port: prueba.dst_port,
                proto: Proto::from_str(&prueba.proto),
                state: parse_state(&prueba.estado),
                ..Default::default()
            };

            let (verdict, _trace) = evaluate(rs, Some(topo), pkt);

            let expected = match prueba.esperado.to_uppercase().as_str() {
                "ACCEPT" => FinalVerdict::Accept,
                "DROP" => FinalVerdict::Drop,
                "REJECT" => FinalVerdict::Reject,
                other => panic!("Esperado desconocido en prueba '{}': {other}", prueba.descripcion),
            };

            assert_eq!(
                verdict,
                expected,
                "Nivel '{}' — prueba '{}': esperado {:?}, obtenido {:?}",
                lvl.id,
                prueba.descripcion,
                expected,
                verdict,
            );
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // Soluciones canónicas por nivel
    // ──────────────────────────────────────────────────────────────────

    fn solution(id: &str) -> &'static [&'static str] {
        match id {
            // Nivel 1: observar — no se necesita ningún comando.
            "01-observar" => &[],

            // Nivel 2: bloquear UDP fantasma en puerto 666.
            "02-primer-guardia" => &[
                "iptables -A INPUT -p udp --dport 666 -j DROP",
            ],

            // Nivel 3: bloquear todo UDP del reino hostil (10.0.0.0/8).
            "03-bloquear-tipo" => &[
                "iptables -A INPUT -p udp -s 10.0.0.0/8 -j DROP",
            ],

            // Nivel 4: política DROP ya pre-configurada + regla ESTABLISHED pre-cargada.
            // El jugador solo añade la excepción SSH. NO usar iptables -F.
            "04-politica-drop" => &[
                "iptables -A INPUT -p tcp --dport 22 -j ACCEPT",
            ],

            // Nivel 5: permitir LAN→DMZ y LAN→WAN; bloquear WAN→LAN y DMZ→LAN.
            "05-tres-portones" => &[
                "iptables -A FORWARD -i eth0 -o eth1 -j ACCEPT",
                "iptables -A FORWARD -i eth0 -o eth2 -j ACCEPT",
            ],

            // Nivel 6: añadir WAN→DMZ solo por HTTP (pre-cargada: ESTABLISHED + LAN→WAN).
            "06-barbacana" => &[
                "iptables -A FORWARD -i eth2 -o eth1 -p tcp --dport 80 -j ACCEPT",
            ],

            // Nivel 7: LAN puede salir a WAN con MASQUERADE (pre-cargada: ESTABLISHED).
            "07-estandarte" => &[
                "iptables -A FORWARD -i eth0 -o eth2 -j ACCEPT",
                "iptables -t nat -A POSTROUTING -o eth2 -j MASQUERADE",
            ],

            // Nivel 8: DNAT puerto 80 al servidor DMZ + FORWARD autorización.
            "08-puerta-secreta" => &[
                "iptables -t nat -A PREROUTING -i eth2 -p tcp --dport 80 -j DNAT --to-destination 10.0.0.5:80",
                "iptables -A FORWARD -i eth2 -o eth1 -p tcp --dport 80 -j ACCEPT",
            ],

            // Nivel 9: examen completo — construir todo desde cero.
            "09-examen-silvia" => &[
                // Ordenanza I: política de denegación total
                "-P INPUT DROP",
                "-P FORWARD DROP",
                "-P OUTPUT DROP",
                // Ordenanza II: tráfico de respuesta autorizado
                "-A INPUT -m state --state ESTABLISHED,RELATED -j ACCEPT",
                "-A FORWARD -m state --state ESTABLISHED,RELATED -j ACCEPT",
                "-A OUTPUT -m state --state ESTABLISHED,RELATED -j ACCEPT",
                // Ordenanza III: Estandarte del Castillo (NAT para LAN y DMZ)
                "-t nat -A POSTROUTING -o eth2 -j MASQUERADE",
                "-A FORWARD -i eth0 -o eth2 -j ACCEPT",
                "-A FORWARD -i eth1 -o eth2 -j ACCEPT",
                // Ordenanza IV: REJECT ICMP desde WAN, ACCEPT desde LAN y DMZ
                "-A INPUT -i eth2 -p icmp -j REJECT",
                "-A INPUT -i eth0 -p icmp -j ACCEPT",
                "-A INPUT -i eth1 -p icmp -j ACCEPT",
                // Ordenanza V: solo SSH saliente al aliado secreto (6.6.6.6)
                "-A OUTPUT -p tcp -d 6.6.6.6 --dport 22 -j ACCEPT",
                // Ordenanza VI: Taller de Pergaminos accesible por HTTP y HTTPS desde WAN (DNAT)
                "-t nat -A PREROUTING -i eth2 -p tcp --dport 80 -j DNAT --to-destination 10.0.0.5:80",
                "-t nat -A PREROUTING -i eth2 -p tcp --dport 443 -j DNAT --to-destination 10.0.0.5:443",
                "-A FORWARD -p tcp -d 10.0.0.5 --dport 80 -j ACCEPT",
                "-A FORWARD -p tcp -d 10.0.0.5 --dport 443 -j ACCEPT",
                // Ordenanza VI (bis): Taller accesible también desde LAN directamente
                "-A FORWARD -i eth0 -o eth1 -p tcp -d 10.0.0.5 --dport 80 -j ACCEPT",
                "-A FORWARD -i eth0 -o eth1 -p tcp -d 10.0.0.5 --dport 443 -j ACCEPT",
                // Ordenanza VII: Biblioteca Secreta (MySQL) solo accesible desde el Taller
                "-A FORWARD -i eth1 -o eth0 -s 10.0.0.5 -d 192.168.1.3 -p tcp --dport 3306 -j ACCEPT",
                // Ordenanza VIII: Mercado de Intercambios (eMule) solo para forasteros
                "-t nat -A PREROUTING -i eth2 -p tcp --dport 4662 -j DNAT --to-destination 192.168.1.20:4662",
                "-t nat -A PREROUTING -i eth2 -p udp --dport 4662 -j DNAT --to-destination 192.168.1.20:4662",
                "-A FORWARD -i eth2 -o eth0 -d 192.168.1.20 -p tcp --dport 4662 -j ACCEPT",
                "-A FORWARD -i eth2 -o eth0 -d 192.168.1.20 -p udp --dport 4662 -j ACCEPT",
                // Ordenanza IX: Sala del Gran Consejo (WEBMIN) solo para el Torreón
                "-A INPUT -i eth0 -p tcp --dport 10000 -j ACCEPT",
                // Ordenanza X: administración remota de la Biblioteca solo desde casa (8.8.8.8)
                "-t nat -A PREROUTING -i eth2 -s 8.8.8.8 -p tcp --dport 22 -j DNAT --to-destination 192.168.1.3:22",
                "-A FORWARD -i eth2 -o eth0 -s 8.8.8.8 -d 192.168.1.3 -p tcp --dport 22 -j ACCEPT",
            ],

            _ => &[],
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // Test principal: todas las soluciones canónicas
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_level_solutions() {
        let levels = load_all();
        assert!(!levels.is_empty(), "No se cargaron niveles");

        for lvl in &levels {
            let cmds = solution(&lvl.id);
            let rs = setup_level(lvl, cmds);
            let topo = topo_from_level(lvl);
            run_pruebas(lvl, &rs, &topo);
        }
    }

    // ──────────────────────────────────────────────────────────────────
    // Regresión: -F antes de la solución rompe el nivel 4
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_level04_flush_breaks() {
        let levels = load_all();
        let lvl = levels
            .iter()
            .find(|l| l.id == "04-politica-drop")
            .expect("Nivel 04-politica-drop no encontrado");

        // Solución incorrecta: -F borra la regla ESTABLISHED pre-cargada
        let bad_solution = &[
            "iptables -F",
            "iptables -A INPUT -p tcp --dport 22 -j ACCEPT",
        ];
        let rs = setup_level(lvl, bad_solution);
        let topo = topo_from_level(lvl);

        // Paquete ESTABLISHED al puerto 80 — con -F, la regla ESTABLISHED desaparece
        // y la política DROP deniega el paquete
        let pkt = Packet {
            src_ip: "203.0.113.5".into(),
            dst_ip: lvl.red.firewall_ip.clone(),
            dst_port: 80,
            proto: Proto::Tcp,
            state: STATE_ESTABLISHED,
            ..Default::default()
        };

        let (verdict, _) = evaluate(&rs, Some(&topo), pkt);
        assert_eq!(
            verdict,
            FinalVerdict::Drop,
            "Con -F + solo SSH, ESTABLISHED/80 debería ser DROP (regresión del flush)"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // Prueba individual: nivel 1 no necesita comandos
    // ──────────────────────────────────────────────────────────────────

    #[test]
    fn test_level01_no_commands_needed() {
        let levels = load_all();
        let lvl = levels
            .iter()
            .find(|l| l.id == "01-observar")
            .expect("Nivel 01-observar no encontrado");

        let rs = setup_level(lvl, &[]);
        let topo = topo_from_level(lvl);

        // Con políticas ACCEPT y sin reglas, todo paquete debería ser ACCEPT
        let pkt = Packet {
            src_ip: "192.168.1.10".into(),
            dst_ip: lvl.red.firewall_ip.clone(),
            dst_port: 80,
            proto: Proto::Tcp,
            state: STATE_NEW,
            ..Default::default()
        };
        let (verdict, _) = evaluate(&rs, Some(&topo), pkt);
        assert_eq!(verdict, FinalVerdict::Accept);
    }
}
