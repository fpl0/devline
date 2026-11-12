package dev.offline.board;

import org.springframework.context.annotation.*;
import org.springframework.security.config.Customizer;
import org.springframework.security.config.annotation.web.builders.HttpSecurity;
import org.springframework.security.core.userdetails.*;
import org.springframework.security.provisioning.InMemoryUserDetailsManager;
import org.springframework.security.web.SecurityFilterChain;

@Configuration
class SecurityConfiguration {
  @Bean
  UserDetailsService users() {
    return new InMemoryUserDetailsManager(
        User.withUsername("alice").password("{noop}devline-local").roles("BUYER").build(),
        User.withUsername("bob").password("{noop}devline-local").roles("APPROVER").build());
  }

  @Bean
  SecurityFilterChain security(HttpSecurity http) throws Exception {
    return http.authorizeHttpRequests(a ->
            a.requestMatchers("/health", "/style.css").permitAll().anyRequest().authenticated())
        .formLogin(Customizer.withDefaults())
        .logout(l -> l.logoutSuccessUrl("/login"))
        .build();
  }
}
