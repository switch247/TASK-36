INSERT INTO zip_city_reference (zip_code, city, state, country) VALUES
('00100', 'Nairobi', 'Nairobi County', 'KE'),
('20100', 'Nakuru', 'Nakuru County', 'KE'),
('40100', 'Kisumu', 'Kisumu County', 'KE'),
('80100', 'Mombasa', 'Mombasa County', 'KE')
ON DUPLICATE KEY UPDATE city = VALUES(city), state = VALUES(state), country = VALUES(country);
