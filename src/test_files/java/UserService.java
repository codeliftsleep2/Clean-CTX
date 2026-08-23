package com.example.demo;

import org.springframework.stereotype.Service;
import org.springframework.beans.factory.annotation.Autowired;
import java.util.List;
import java.util.ArrayList;

/**
 * Test fixture: Spring Boot service for Java/Spring meta-layer tests.
 */
@Service
public class UserService {

    private final UserRepository userRepository;

    @Autowired
    public UserService(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    public List<UserDto> findAll() {
        return userRepository.findAll();
    }

    public UserDto findById(Long id) {
        return userRepository.findById(id);
    }

    public UserDto create(CreateUserRequest request) {
        return userRepository.save(request);
    }

    public UserDto update(Long id, UpdateUserRequest request) {
        return userRepository.update(id, request);
    }

    public void delete(Long id) {
        userRepository.delete(id);
    }
}