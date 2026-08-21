import { beforeEach, describe, expect, it } from 'vitest';
import { TestBed } from '@angular/core/testing';

import { sampleWithNewData, sampleWithRequiredData } from '../conversation.test-samples';

import { ConversationFormService } from './conversation-form.service';

describe('Conversation Form Service', () => {
  let service: ConversationFormService;

  beforeEach(() => {
    service = TestBed.inject(ConversationFormService);
  });

  describe('Service methods', () => {
    describe('createConversationFormGroup', () => {
      it('should create a new form with FormControl', () => {
        const formGroup = service.createConversationFormGroup();

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            title: expect.any(Object),
            summary: expect.any(Object),
            metadata: expect.any(Object),
            typeChat: expect.any(Object),
            createdAt: expect.any(Object),
            lastMessageAt: expect.any(Object),
          }),
        );
      });

      it('passing IConversation should create a new form with FormGroup', () => {
        const formGroup = service.createConversationFormGroup(sampleWithRequiredData);

        expect(formGroup.controls).toEqual(
          expect.objectContaining({
            id: expect.any(Object),
            userId: expect.any(Object),
            title: expect.any(Object),
            summary: expect.any(Object),
            metadata: expect.any(Object),
            typeChat: expect.any(Object),
            createdAt: expect.any(Object),
            lastMessageAt: expect.any(Object),
          }),
        );
      });
    });

    describe('getConversation', () => {
      it('should return NewConversation for default Conversation initial value', () => {
        const formGroup = service.createConversationFormGroup(sampleWithNewData);

        const conversation = service.getConversation(formGroup);

        expect(conversation).toMatchObject(sampleWithNewData);
      });

      it('should return NewConversation for empty Conversation initial value', () => {
        const formGroup = service.createConversationFormGroup();

        const conversation = service.getConversation(formGroup);

        expect(conversation).toMatchObject({});
      });

      it('should return IConversation', () => {
        const formGroup = service.createConversationFormGroup(sampleWithRequiredData);

        const conversation = service.getConversation(formGroup);

        expect(conversation).toMatchObject(sampleWithRequiredData);
      });
    });

    describe('resetForm', () => {
      it('passing IConversation should not enable id FormControl', () => {
        const formGroup = service.createConversationFormGroup();
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, sampleWithRequiredData);

        expect(formGroup.controls.id.disabled).toBe(true);
      });

      it('passing NewConversation should disable id FormControl', () => {
        const formGroup = service.createConversationFormGroup(sampleWithRequiredData);
        expect(formGroup.controls.id.disabled).toBe(true);

        service.resetForm(formGroup, { id: null });

        expect(formGroup.controls.id.disabled).toBe(true);
      });
    });
  });
});
